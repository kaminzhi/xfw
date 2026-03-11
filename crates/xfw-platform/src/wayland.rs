use std::os::fd::{AsFd, AsRawFd};

use wayland_client::Connection;

use crate::error::PlatformResult;

pub struct WaylandConnectionInner {
    pub connection: Connection,
}

pub struct WaylandConnection {
    pub display: String,
    pub socket_name: Option<String>,
    pub inner: Option<WaylandConnectionInner>,
}

impl WaylandConnection {
    pub fn new() -> PlatformResult<Self> {
        let wayland_display =
            std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());

        let connection = match Connection::connect_to_env() {
            Ok(conn) => conn,
            Err(e) => {
                tracing::warn!(error = %e, "failed to connect to Wayland, running in headless mode");
                return Ok(Self {
                    display: wayland_display,
                    socket_name: None,
                    inner: None,
                });
            }
        };

        let fd = connection.as_fd().as_raw_fd();

        tracing::info!(display = %wayland_display, fd = fd, "connected to Wayland");

        Ok(Self {
            display: wayland_display,
            socket_name: None,
            inner: Some(WaylandConnectionInner { connection }),
        })
    }

    pub fn is_connected(&self) -> bool {
        self.inner.is_some()
    }

    pub fn get_fd(&self) -> Option<i32> {
        self.inner
            .as_ref()
            .map(|i| i.connection.as_fd().as_raw_fd())
    }
}

impl Default for WaylandConnection {
    fn default() -> Self {
        Self::new().expect("Failed to create Wayland connection")
    }
}
