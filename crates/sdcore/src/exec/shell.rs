use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Sender};
use std::thread;
use std::time::{Duration, Instant};

use super::{ExecEvent, ExecOutput, ExecRequest, ExecResult};

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const TERMINATE_GRACE: Duration = Duration::from_secs(5);

#[derive(Default)]
pub struct ShellExecutor;

impl ShellExecutor {
    pub fn run<F>(&self, request: ExecRequest<'_>, mut emit: F) -> Result<ExecResult, String>
    where
        F: FnMut(ExecEvent),
    {
        let command = request.command.to_string();
        emit(ExecEvent::Started {
            command: command.clone(),
        });

        let mut process = Command::new("sh");
        process
            .arg("-lc")
            .arg(request.command)
            .current_dir(request.working_dir)
            .envs(request.envs.iter().copied())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            process.process_group(0);
        }

        let mut child = process
            .spawn()
            .map_err(|err| format!("failed to start command `{command}`: {err}"))?;

        let (sender, receiver) = mpsc::channel();
        if let Some(stdout) = child.stdout.take() {
            spawn_reader(stdout, ExecStream::Stdout, sender.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_reader(stderr, ExecStream::Stderr, sender.clone());
        }
        drop(sender);

        let started_at = Instant::now();
        loop {
            while let Ok(event) = receiver.try_recv() {
                emit(event);
            }

            if let Some(status) = child
                .try_wait()
                .map_err(|err| format!("failed to wait command `{command}`: {err}"))?
            {
                for event in receiver {
                    emit(event);
                }
                let result = ExecResult {
                    success: status.success(),
                    status: status.to_string(),
                };
                emit(ExecEvent::Finished {
                    command,
                    success: result.success,
                    status: result.status.clone(),
                });
                return Ok(result);
            }

            if let Some(timeout) = request.timeout {
                if started_at.elapsed() >= timeout {
                    let message = format!("command timed out after {} seconds", timeout.as_secs());
                    emit(ExecEvent::Stderr { content: message });
                    terminate_child(&mut child);
                    drain_receiver(&receiver, &mut emit);
                    let result = ExecResult {
                        success: false,
                        status: format!("timeout after {} seconds", timeout.as_secs()),
                    };
                    emit(ExecEvent::Finished {
                        command,
                        success: false,
                        status: result.status.clone(),
                    });
                    return Ok(result);
                }
            }

            let wait_for = request
                .timeout
                .map(|timeout| {
                    timeout
                        .saturating_sub(started_at.elapsed())
                        .min(POLL_INTERVAL)
                })
                .filter(|duration| !duration.is_zero())
                .unwrap_or(POLL_INTERVAL);
            if let Ok(event) = receiver.recv_timeout(wait_for) {
                emit(event);
            }
        }
    }

    pub fn run_capture(&self, request: ExecRequest<'_>) -> Result<ExecOutput, String> {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();
        let result = self.run(request, |event| match event {
            ExecEvent::Stdout { content } => stdout.push(content),
            ExecEvent::Stderr { content } => stderr.push(content),
            ExecEvent::Started { .. } | ExecEvent::Finished { .. } => {}
        })?;
        Ok(ExecOutput {
            success: result.success,
            status: result.status,
            stdout: stdout.join("\n"),
            stderr: stderr.join("\n"),
        })
    }
}

#[derive(Clone, Copy)]
enum ExecStream {
    Stdout,
    Stderr,
}

fn spawn_reader<R>(reader: R, stream: ExecStream, sender: Sender<ExecEvent>)
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let reader = BufReader::new(reader);
        for line in reader.lines() {
            let Ok(line) = line else {
                break;
            };
            let event = match stream {
                ExecStream::Stdout => ExecEvent::Stdout { content: line },
                ExecStream::Stderr => ExecEvent::Stderr { content: line },
            };
            if sender.send(event).is_err() {
                break;
            }
        }
    });
}

fn drain_receiver<F>(receiver: &mpsc::Receiver<ExecEvent>, emit: &mut F)
where
    F: FnMut(ExecEvent),
{
    while let Ok(event) = receiver.recv_timeout(Duration::from_millis(20)) {
        emit(event);
    }
}

fn terminate_child(child: &mut Child) {
    terminate_process(child);
    if wait_for_exit(child, TERMINATE_GRACE) {
        return;
    }
    kill_process(child);
    let _ = child.wait();
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> bool {
    let started_at = Instant::now();
    loop {
        if matches!(child.try_wait(), Ok(Some(_))) {
            return true;
        }
        if started_at.elapsed() >= timeout {
            return false;
        }
        thread::sleep(POLL_INTERVAL);
    }
}

#[cfg(unix)]
fn terminate_process(child: &mut Child) {
    signal_process_group(child, SIGTERM);
}

#[cfg(not(unix))]
fn terminate_process(child: &mut Child) {
    let _ = child.kill();
}

#[cfg(unix)]
fn kill_process(child: &mut Child) {
    signal_process_group(child, SIGKILL);
}

#[cfg(not(unix))]
fn kill_process(child: &mut Child) {
    let _ = child.kill();
}

#[cfg(unix)]
fn signal_process_group(child: &mut Child, signal: i32) {
    let process_group = -(child.id() as i32);
    let result = unsafe { kill(process_group, signal) };
    if result != 0 {
        let _ = child.kill();
    }
}

#[cfg(unix)]
const SIGTERM: i32 = 15;
#[cfg(unix)]
const SIGKILL: i32 = 9;

#[cfg(unix)]
unsafe extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}
