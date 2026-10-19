use std::time::{Duration, Instant};

use sdcore::exec::{ExecEvent, ExecRequest, ShellExecutor};

#[test]
fn shell_executor_emits_stdout_and_stderr_events() {
    let executor = ShellExecutor;
    let cwd = std::env::temp_dir();
    let mut events = Vec::new();

    let result = executor
        .run(
            ExecRequest::new(&cwd, "printf 'hello\\n'; printf 'warn\\n' >&2"),
            |event| events.push(event),
        )
        .unwrap();

    assert!(result.success);
    assert!(events
        .iter()
        .any(|event| matches!(event, ExecEvent::Stdout { content } if content == "hello")));
    assert!(events
        .iter()
        .any(|event| matches!(event, ExecEvent::Stderr { content } if content == "warn")));
}

#[test]
fn shell_executor_times_out_long_running_commands() {
    let executor = ShellExecutor;
    let cwd = std::env::temp_dir();
    let started_at = Instant::now();
    let mut events = Vec::new();

    let result = executor
        .run(
            ExecRequest::new(&cwd, "sleep 3; printf 'done\\n'")
                .with_timeout(Duration::from_secs(1)),
            |event| events.push(event),
        )
        .unwrap();

    assert!(!result.success);
    assert!(result.status.contains("timeout"));
    assert!(started_at.elapsed() < Duration::from_secs(3));
    assert!(events.iter().any(
        |event| matches!(event, ExecEvent::Stderr { content } if content.contains("timed out"))
    ));
}

#[test]
fn shell_executor_capture_supports_envs_and_streams() {
    let executor = ShellExecutor;
    let cwd = std::env::temp_dir();

    let output = executor
        .run_capture(
            ExecRequest::new(&cwd, "printf \"$SDCORE_EXEC_TEST\"; printf 'warn' >&2")
                .with_env("SDCORE_EXEC_TEST", "hello"),
        )
        .unwrap();

    assert!(output.success);
    assert_eq!(output.stdout, "hello");
    assert_eq!(output.stderr, "warn");
}
