//! Managed, long-lived subprocesses with bounded output subscriptions.
//!
//! Commands use separate executable and argument values, without an implicit
//! shell. Dropping the last handle requests shutdown. On Unix, shutdown also
//! terminates descendants in the process group.

mod managed;
mod types;

pub use managed::ProcessHandle;
pub use types::{ProcessEvent, ProcessExit, ProcessRequest, ProcessState, ProcessStream};
