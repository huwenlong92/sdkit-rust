use std::io;
use std::process::Stdio;
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
use tokio::sync::{broadcast, watch};
use tokio::task::JoinHandle;

use super::{ProcessEvent, ProcessExit, ProcessRequest, ProcessState, ProcessStream};

const EVENT_CAPACITY: usize = 256;
const OUTPUT_CHUNK_SIZE: usize = 8192;
const OUTPUT_DRAIN_TIMEOUT: Duration = Duration::from_secs(1);

/// A clonable handle to one managed subprocess.
///
/// The last handle's drop requests graceful shutdown. Keep the Tokio runtime
/// alive to let cleanup complete; call `stop().await` during application shutdown.
/// Dropping an event receiver does not stop the process. Output subscribers are
/// bounded and may receive `RecvError::Lagged`; state remains available separately.
#[derive(Clone)]
pub struct ProcessHandle {
    pid: u32,
    stop: watch::Sender<bool>,
    state: watch::Receiver<ProcessState>,
    events: broadcast::Sender<ProcessEvent>,
}

impl ProcessHandle {
    /// Requires an active Tokio runtime. The returned subscription captures
    /// events from startup, including output from processes that exit immediately.
    pub fn spawn(request: ProcessRequest) -> io::Result<(Self, broadcast::Receiver<ProcessEvent>)> {
        tokio::runtime::Handle::try_current()
            .map_err(|_| io::Error::other("managed processes require a Tokio runtime"))?;
        let mut command = Command::new(&request.program);
        if request.clear_env {
            command.env_clear();
        }
        command
            .args(&request.args)
            .envs(request.envs)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        if let Some(path) = request.working_dir {
            command.current_dir(path);
        }
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn()?;
        let pid = child.id().expect("newly spawned process has a PID");
        let (events, subscription) = broadcast::channel(EVENT_CAPACITY);
        let (state_tx, state) = watch::channel(ProcessState::Running);
        let (stop, stop_rx) = watch::channel(false);
        let _ = events.send(ProcessEvent::Started { pid });

        let stdout = child
            .stdout
            .take()
            .map(|reader| spawn_reader(reader, ProcessStream::Stdout, events.clone()));
        let stderr = child
            .stderr
            .take()
            .map(|reader| spawn_reader(reader, ProcessStream::Stderr, events.clone()));
        tokio::spawn(supervise(
            child,
            pid,
            request.stop_timeout,
            stop_rx,
            state_tx,
            events.clone(),
            [stdout, stderr],
        ));
        Ok((
            Self {
                pid,
                stop,
                state,
                events,
            },
            subscription,
        ))
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    pub fn state(&self) -> ProcessState {
        self.state.borrow().clone()
    }

    pub fn subscribe_state(&self) -> watch::Receiver<ProcessState> {
        self.state.clone()
    }

    /// Subscribes to future events. Historical output is not stored here.
    pub fn subscribe(&self) -> broadcast::Receiver<ProcessEvent> {
        self.events.subscribe()
    }

    pub async fn wait(&self) -> Result<ProcessExit, String> {
        let mut state = self.state.clone();
        loop {
            match state.borrow_and_update().clone() {
                ProcessState::Exited(exit) => return Ok(exit),
                ProcessState::Failed(message) => return Err(message),
                ProcessState::Running | ProcessState::Stopping => {}
            }
            state
                .changed()
                .await
                .map_err(|_| "process supervisor closed".to_string())?;
        }
    }

    /// Idempotent and safe to call concurrently. Cancellation of this future
    /// does not cancel shutdown after the stop request has been sent.
    pub async fn stop(&self) -> Result<ProcessExit, String> {
        let _ = self.stop.send(true);
        self.wait().await
    }
}

async fn supervise(
    mut child: Child,
    pid: u32,
    stop_timeout: Duration,
    mut stop: watch::Receiver<bool>,
    state: watch::Sender<ProcessState>,
    events: broadcast::Sender<ProcessEvent>,
    readers: [Option<JoinHandle<()>>; 2],
) {
    let result = tokio::select! {
        result = child.wait() => result,
        // A stop request or the last handle dropping triggers shutdown.
        _ = stop.changed() => {
            state.send_replace(ProcessState::Stopping);
            let _ = events.send(ProcessEvent::Stopping);
            stop_child(&mut child, pid, stop_timeout).await
        }
    };
    // A descendant may keep running or hold output pipes after its parent exits.
    // The managed process owns its Unix process group, including on natural exit.
    #[cfg(unix)]
    let _ = signal_group(pid, SIGKILL);
    for mut reader in readers.into_iter().flatten() {
        if tokio::time::timeout(OUTPUT_DRAIN_TIMEOUT, &mut reader)
            .await
            .is_err()
        {
            reader.abort();
            let _ = reader.await;
        }
    }
    match result {
        Ok(status) => {
            let exit = ProcessExit {
                success: status.success(),
                code: status.code(),
                status: status.to_string(),
            };
            let _ = events.send(ProcessEvent::Exited(exit.clone()));
            state.send_replace(ProcessState::Exited(exit));
        }
        Err(error) => {
            let message = error.to_string();
            let _ = events.send(ProcessEvent::Failed(message.clone()));
            state.send_replace(ProcessState::Failed(message));
        }
    }
}

fn spawn_reader<R>(
    mut reader: R,
    stream: ProcessStream,
    events: broadcast::Sender<ProcessEvent>,
) -> JoinHandle<()>
where
    R: AsyncRead + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        let mut buffer = [0; OUTPUT_CHUNK_SIZE];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(length) => {
                    let _ = events.send(ProcessEvent::Output {
                        stream,
                        bytes: buffer[..length].to_vec(),
                    });
                }
                Err(error) => {
                    let _ = events.send(ProcessEvent::ReadError {
                        stream,
                        message: error.to_string(),
                    });
                    break;
                }
            }
        }
    })
}

async fn stop_child(
    child: &mut Child,
    pid: u32,
    timeout: Duration,
) -> io::Result<std::process::ExitStatus> {
    #[cfg(unix)]
    {
        if signal_group(pid, SIGTERM).is_err() {
            // If signaling fails or the process already exited, try the child
            // directly and still collect its exit status with wait().
            let _ = child.start_kill();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        child.start_kill()?;
    }
    match tokio::time::timeout(timeout, child.wait()).await {
        Ok(result) => result,
        Err(_) => {
            #[cfg(unix)]
            let _ = signal_group(pid, SIGKILL);
            child.start_kill()?;
            child.wait().await
        }
    }
}

#[cfg(unix)]
const SIGTERM: i32 = 15;
#[cfg(unix)]
const SIGKILL: i32 = 9;

#[cfg(unix)]
fn signal_group(pid: u32, signal: i32) -> io::Result<()> {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    if unsafe { kill(-(pid as i32), signal) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}
