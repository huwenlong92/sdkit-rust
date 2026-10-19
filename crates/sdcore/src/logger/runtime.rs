use chrono::Local;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use super::{LogConfig, LogLevel};

static LOGGER: OnceLock<Mutex<Option<Logger>>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Logger {
    config: LogConfig,
    file_path: PathBuf,
}

impl Logger {
    pub fn new(config: LogConfig) -> Result<Self, String> {
        let file_path = log_file_path(&config);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        Ok(Self { config, file_path })
    }

    pub fn named(&self, name: impl Into<String>) -> Result<Self, String> {
        Self::new(LogConfig {
            name: name.into(),
            ..self.config.clone()
        })
    }

    pub fn debug(&self, message: impl AsRef<str>) {
        self.emit(LogLevel::Debug, message.as_ref()).ok();
    }

    pub fn info(&self, message: impl AsRef<str>) {
        self.emit(LogLevel::Info, message.as_ref()).ok();
    }

    pub fn warn(&self, message: impl AsRef<str>) {
        self.emit(LogLevel::Warn, message.as_ref()).ok();
    }

    pub fn error(&self, message: impl AsRef<str>) {
        self.emit(LogLevel::Error, message.as_ref()).ok();
    }

    pub fn emit(&self, level: LogLevel, message: &str) -> Result<(), String> {
        if level < self.config.level {
            return Ok(());
        }
        let line = format_line(level, message);
        if self.config.console {
            write_console(level, &line);
        }
        self.write_file(&line)
    }

    fn write_file(&self, line: &str) -> Result<(), String> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_path)
            .and_then(|mut file| file.write_all(line.as_bytes()))
            .map_err(|e| e.to_string())
    }
}

pub fn init(config: LogConfig) -> Result<(), String> {
    let logger = Logger::new(config)?;
    let mut guard = global().lock().map_err(|e| e.to_string())?;
    *guard = Some(logger);
    Ok(())
}

pub fn named(name: impl Into<String>) -> Result<Logger, String> {
    let name = name.into();
    let guard = global().lock().map_err(|e| e.to_string())?;
    let config = if let Some(logger) = guard.as_ref() {
        LogConfig {
            name,
            ..logger.config.clone()
        }
    } else {
        LogConfig::default().name(name)
    };
    Logger::new(config)
}

pub fn debug(message: impl AsRef<str>) {
    emit_global(LogLevel::Debug, message.as_ref());
}

pub fn info(message: impl AsRef<str>) {
    emit_global(LogLevel::Info, message.as_ref());
}

pub fn warn(message: impl AsRef<str>) {
    emit_global(LogLevel::Warn, message.as_ref());
}

pub fn error(message: impl AsRef<str>) {
    emit_global(LogLevel::Error, message.as_ref());
}

pub fn sync() {}

fn emit_global(level: LogLevel, message: &str) {
    if let Ok(guard) = global().lock() {
        if let Some(logger) = guard.as_ref() {
            if let Err(err) = logger.emit(level, message) {
                write_console(LogLevel::Error, &format_line(LogLevel::Error, &err));
            }
            return;
        }
    }
    write_console(level, &format_line(level, message));
}

fn global() -> &'static Mutex<Option<Logger>> {
    LOGGER.get_or_init(|| Mutex::new(None))
}

fn log_file_path(config: &LogConfig) -> PathBuf {
    config
        .root_dir
        .join(&config.name)
        .join(format!("{}.log", config.name))
}

fn format_line(level: LogLevel, message: &str) -> String {
    format!(
        "[{}] {:<5} {}\n",
        now_string(),
        level.as_str(),
        message.trim_end()
    )
}

fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn write_console(level: LogLevel, line: &str) {
    if level >= LogLevel::Warn {
        eprint!("{line}");
    } else {
        print!("{line}");
    }
}
