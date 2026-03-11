use thiserror::Error;

#[derive(Error, Debug)]
pub enum PlatformError {
    #[error("Wayland connection failed: {0}")]
    WaylandConnection(String),
    #[error("Event loop error: {0}")]
    EventLoop(String),
    #[error("File descriptor error: {0}")]
    FdError(String),
}

pub type PlatformResult<T> = std::result::Result<T, PlatformError>;
