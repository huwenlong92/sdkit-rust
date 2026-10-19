use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

/// Configuration for one process. Arguments and environment values are never
/// included in lifecycle events, so callers can keep credentials out of logs.
#[derive(Debug)]
pub struct ProcessRequest {
    pub program: OsString,
    pub args: Vec<OsString>,
    pub working_dir: Option<PathBuf>,
    pub envs: Vec<(OsString, OsString)>,
    pub clear_env: bool,
    pub stop_timeout: Duration,
}

impl ProcessRequest {
    pub fn new(program: impl Into<OsString>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            working_dir: None,
            envs: Vec::new(),
            clear_env: false,
            stop_timeout: Duration::from_secs(5),
        }
    }

    pub fn with_arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn with_working_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(path.into());
        self
    }

    pub fn with_env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.envs.push((key.into(), value.into()));
        self
    }

    /// Avoid inheriting credentials or connector configuration from the parent.
    pub fn with_clear_env(mut self) -> Self {
        self.clear_env = true;
        self
    }

    /// Isolate a tool's user configuration while retaining OS runtime variables.
    /// The caller creates the home directory. No parent credentials or proxies
    /// are inherited. A later call replaces all home variables together.
    pub fn with_isolated_home(mut self, home: impl Into<PathBuf>) -> Self {
        let home = home.into();
        self.clear_env = true;
        self.envs.retain(|(key, _)| {
            ![
                "HOME",
                "USERPROFILE",
                "APPDATA",
                "LOCALAPPDATA",
                "TEMP",
                "TMP",
                "PATH",
                "SystemRoot",
                "WINDIR",
            ]
            .iter()
            .any(|name| key.as_encoded_bytes().eq_ignore_ascii_case(name.as_bytes()))
        });
        self.envs
            .push(("HOME".into(), home.clone().into_os_string()));
        if let Some(path) = std::env::var_os("PATH") {
            self.envs.push(("PATH".into(), path));
        }
        #[cfg(windows)]
        {
            for name in ["USERPROFILE", "APPDATA", "LOCALAPPDATA", "TEMP", "TMP"] {
                self.envs.push((name.into(), home.clone().into_os_string()));
            }
            for name in ["SystemRoot", "WINDIR"] {
                if let Some(value) = std::env::var_os(name) {
                    self.envs.push((name.into(), value));
                }
            }
        }
        self
    }

    pub fn with_stop_timeout(mut self, timeout: Duration) -> Self {
        self.stop_timeout = timeout;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessExit {
    pub success: bool,
    pub code: Option<i32>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessState {
    Running,
    Stopping,
    Exited(ProcessExit),
    /// Waiting or stopping failed; this is different from a nonzero exit code.
    Failed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessEvent {
    Started {
        pid: u32,
    },
    /// Raw chunks of up to 8 KiB, not necessarily complete lines or UTF-8.
    Output {
        stream: ProcessStream,
        bytes: Vec<u8>,
    },
    ReadError {
        stream: ProcessStream,
        message: String,
    },
    Stopping,
    Exited(ProcessExit),
    Failed(String),
}
