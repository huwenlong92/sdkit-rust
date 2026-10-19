pub mod app;
#[cfg(feature = "auth")]
pub mod auth;
pub mod config;
pub mod db;
pub mod exec;
pub mod http;
pub mod logger;
pub mod paths;
pub mod process;
pub mod reporter;
pub mod sse;
pub mod static_files;
pub mod task;

pub use axum;
#[cfg(any(feature = "sqlite", feature = "postgres"))]
pub use sqlx;
pub use tokio;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}
