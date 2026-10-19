use std::path::Path;
use std::time::Duration;

pub struct ExecRequest<'a> {
    pub working_dir: &'a Path,
    pub command: &'a str,
    pub timeout: Option<Duration>,
    pub envs: Vec<(&'a str, &'a str)>,
}

impl<'a> ExecRequest<'a> {
    pub fn new(working_dir: &'a Path, command: &'a str) -> Self {
        Self {
            working_dir,
            command,
            timeout: None,
            envs: Vec::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn with_env(mut self, key: &'a str, value: &'a str) -> Self {
        self.envs.push((key, value));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecResult {
    pub success: bool,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    pub success: bool,
    pub status: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecEvent {
    Started {
        command: String,
    },
    Stdout {
        content: String,
    },
    Stderr {
        content: String,
    },
    Finished {
        command: String,
        success: bool,
        status: String,
    },
}
