#![cfg(unix)]

use std::time::Duration;

use sdcore::process::{ProcessEvent, ProcessHandle, ProcessRequest, ProcessState, ProcessStream};
use tokio::sync::broadcast;

const TEST_TIMEOUT: Duration = Duration::from_secs(5);

fn shell(script: &str) -> ProcessRequest {
    ProcessRequest::new("/bin/sh")
        .with_arg("-c")
        .with_arg(script)
}

async fn read_ready(receiver: &mut broadcast::Receiver<ProcessEvent>) {
    tokio::time::timeout(TEST_TIMEOUT, async {
        let mut bytes = Vec::new();
        loop {
            match receiver.recv().await.unwrap() {
                ProcessEvent::Output {
                    stream: ProcessStream::Stdout,
                    bytes: chunk,
                } => {
                    bytes.extend(chunk);
                    if bytes.windows(5).any(|window| window == b"ready") {
                        break;
                    }
                }
                ProcessEvent::Exited(exit) => panic!("exited before ready: {exit:?}"),
                _ => {}
            }
        }
    })
    .await
    .expect("process did not become ready");
}

#[tokio::test]
async fn managed_process_preserves_arguments_environment_and_working_directory() {
    let cwd = std::env::temp_dir();
    let literal = "a space; $(this-must-not-execute)";
    let request =
        shell("printf '%s\n%s\n' \"$1\" \"$SDCORE_PROCESS_TEST\"; pwd; printf warn >&2; exit 7")
            .with_arg("test")
            .with_arg(literal)
            .with_env("SDCORE_PROCESS_TEST", "environment value")
            .with_working_dir(&cwd);
    let (process, mut events) = ProcessHandle::spawn(request).unwrap();
    let exit = tokio::time::timeout(TEST_TIMEOUT, process.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(!exit.success);
    assert_eq!(exit.code, Some(7));
    assert_eq!(process.state(), ProcessState::Exited(exit.clone()));

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut started = false;
    while let Ok(event) = events.try_recv() {
        match event {
            ProcessEvent::Started { pid } => {
                assert_eq!(pid, process.pid());
                started = true;
            }
            ProcessEvent::Output {
                stream: ProcessStream::Stdout,
                bytes,
            } => stdout.extend(bytes),
            ProcessEvent::Output {
                stream: ProcessStream::Stderr,
                bytes,
            } => stderr.extend(bytes),
            ProcessEvent::Exited(actual) => assert_eq!(actual, exit),
            other => panic!("unexpected event: {other:?}"),
        }
    }
    assert!(started);
    let stdout = String::from_utf8(stdout).unwrap();
    let mut lines = stdout.lines();
    assert_eq!(lines.next(), Some(literal));
    assert_eq!(lines.next(), Some("environment value"));
    assert_eq!(
        std::fs::canonicalize(lines.next().unwrap()).unwrap(),
        std::fs::canonicalize(cwd).unwrap()
    );
    assert_eq!(stderr, b"warn");
}

#[tokio::test]
async fn managed_process_allows_graceful_and_concurrent_stop() {
    let (process, mut events) = ProcessHandle::spawn(shell(
        "trap 'printf stopped; exit 0' TERM; printf ready; while :; do sleep 0.05; done",
    ))
    .unwrap();
    read_ready(&mut events).await;
    assert_eq!(process.state(), ProcessState::Running);
    let (first, second) = tokio::time::timeout(TEST_TIMEOUT, async {
        tokio::join!(process.stop(), process.stop())
    })
    .await
    .unwrap();
    assert!(first.as_ref().unwrap().success);
    assert_eq!(first, second);
    assert_eq!(process.stop().await, first);
    let mut output = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let ProcessEvent::Output {
            stream: ProcessStream::Stdout,
            bytes,
        } = event
        {
            output.extend(bytes);
        }
    }
    assert!(output.windows(7).any(|window| window == b"stopped"));
}

#[tokio::test]
async fn managed_process_force_kills_after_grace_period() {
    let (process, mut events) = ProcessHandle::spawn(
        shell("trap '' TERM; printf ready; while :; do sleep 1; done")
            .with_stop_timeout(Duration::from_millis(100)),
    )
    .unwrap();
    read_ready(&mut events).await;
    let exit = tokio::time::timeout(TEST_TIMEOUT, process.stop())
        .await
        .unwrap()
        .unwrap();
    assert!(!exit.success);
    assert_eq!(exit.code, None);
}

#[tokio::test]
async fn cancelling_stop_wait_does_not_cancel_shutdown() {
    let (process, mut events) = ProcessHandle::spawn(
        shell("trap '' TERM; printf ready; while :; do sleep 1; done")
            .with_stop_timeout(Duration::from_millis(100)),
    )
    .unwrap();
    read_ready(&mut events).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(1), process.stop())
            .await
            .is_err()
    );
    let exit = tokio::time::timeout(TEST_TIMEOUT, process.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(!exit.success);
    assert_eq!(exit.code, None);
}

#[tokio::test]
async fn dropping_last_handle_stops_process_but_dropping_subscription_does_not() {
    let (process, mut events) = ProcessHandle::spawn(shell(
        "trap 'exit 0' TERM; printf ready; while :; do sleep 0.05; done",
    ))
    .unwrap();
    read_ready(&mut events).await;
    drop(events);
    let second_handle = process.clone();
    let mut state = process.subscribe_state();
    drop(process);
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(second_handle.state(), ProcessState::Running);
    drop(second_handle);
    tokio::time::timeout(TEST_TIMEOUT, async {
        loop {
            if matches!(*state.borrow_and_update(), ProcessState::Exited(_)) {
                break;
            }
            state.changed().await.unwrap();
        }
    })
    .await
    .expect("last handle did not trigger cleanup");
}

#[tokio::test]
async fn slow_output_subscriber_does_not_block_process_or_exit() {
    let (process, mut events) =
        ProcessHandle::spawn(shell("dd if=/dev/zero bs=8192 count=512 2>/dev/null")).unwrap();
    let exit = tokio::time::timeout(TEST_TIMEOUT, process.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(exit.success);
    assert!(matches!(
        events.try_recv(),
        Err(broadcast::error::TryRecvError::Lagged(_))
    ));
    while let Ok(event) = events.try_recv() {
        if let ProcessEvent::Output { bytes, .. } = event {
            assert!(bytes.len() <= 8192);
        }
    }
}

async fn process_is_alive(pid: u32) -> bool {
    let output = tokio::process::Command::new("ps")
        .args(["-o", "stat=", "-p", &pid.to_string()])
        .output()
        .await
        .unwrap();
    let state = String::from_utf8_lossy(&output.stdout);
    output.status.success() && !state.trim().is_empty() && !state.trim().starts_with('Z')
}

#[tokio::test]
async fn managed_process_cleans_descendants_even_after_parent_exits_naturally() {
    let (process, mut events) = ProcessHandle::spawn(shell(
        "(trap '' TERM; while :; do sleep 1; done) & printf '%s\n' \"$!\"; exit 0",
    ))
    .unwrap();
    let exit = tokio::time::timeout(TEST_TIMEOUT, process.wait())
        .await
        .unwrap()
        .unwrap();
    assert!(exit.success);
    let mut stdout = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let ProcessEvent::Output {
            stream: ProcessStream::Stdout,
            bytes,
        } = event
        {
            stdout.extend(bytes);
        }
    }
    let pid: u32 = String::from_utf8(stdout).unwrap().trim().parse().unwrap();
    tokio::time::timeout(TEST_TIMEOUT, async {
        while process_is_alive(pid).await {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("managed descendant is still running");
}

#[tokio::test]
async fn spawn_failure_is_returned_without_a_running_handle() {
    let request = ProcessRequest::new("/nonexistent-sdcore-process-test/program");
    match ProcessHandle::spawn(request) {
        Err(error) => assert_eq!(
            error.kind(),
            std::io::ErrorKind::NotFound,
            "spawn error: {error:?}"
        ),
        Ok((process, _)) => {
            let exit = process.wait().await;
            panic!("missing executable unexpectedly spawned: {exit:?}");
        }
    }
}

#[test]
fn managed_process_requires_runtime_instead_of_panicking() {
    assert!(
        matches!(ProcessHandle::spawn(shell("exit 0")), Err(error) if error.to_string().contains("Tokio runtime"))
    );
}

#[tokio::test]
async fn managed_process_can_clear_inherited_environment() {
    let request = ProcessRequest::new("/usr/bin/env")
        .with_clear_env()
        .with_env("SDCORE_ISOLATED", "yes");
    let (handle, mut events) = ProcessHandle::spawn(request).unwrap();
    assert!(handle.wait().await.unwrap().success);
    let mut output = Vec::new();
    while let Ok(event) = events.try_recv() {
        if let ProcessEvent::Output { bytes, .. } = event {
            output.extend(bytes);
        }
    }
    assert_eq!(String::from_utf8(output).unwrap(), "SDCORE_ISOLATED=yes\n");
}
