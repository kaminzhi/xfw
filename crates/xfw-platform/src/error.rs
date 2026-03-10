use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Wayland connection error: {0}")]
    Wayland(String),

    #[error("Surface error: {0}")]
    Surface(String),

    #[error("Buffer error: {0}")]
    Buffer(String),

    #[error("Event loop error: {0}")]
    EventLoop(String),

    #[error("Protocol not supported: {0}")]
    ProtocolNotSupported(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Render error: {0}")]
    Render(String),
}

pub type Result<T> = std::result::Result<T, PlatformError>;
