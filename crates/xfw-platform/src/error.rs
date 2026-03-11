use std::fmt;

pub type Result<T> = std::result::Result<T, anyhow::Error>;

pub fn wayland_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Wayland connection error: {}", msg.into())
}

pub fn surface_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Surface error: {}", msg.into())
}

pub fn buffer_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Buffer error: {}", msg.into())
}

pub fn event_loop_error(msg: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Event loop error: {}", msg.into())
}

pub fn protocol_not_supported(name: impl Into<String>) -> anyhow::Error {
    anyhow::anyhow!("Protocol not supported: {}", name.into())
}
