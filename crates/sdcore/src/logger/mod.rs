mod config;
mod runtime;

pub use config::{LogConfig, LogLevel};
pub use runtime::{debug, error, info, init, named, sync, warn, Logger};
