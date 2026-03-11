use std::sync::Arc;

use anyhow::Result;
use mio::Interest;
use mio::Token;

use crate::event::{PlatformEvent, PlatformEventHandler};
use crate::event_loop::EventLoop;
use crate::wayland::WaylandConnection;

pub struct PlatformSurface {
    wayland: WaylandConnection,
    event_loop: EventLoop,
    running: bool,
}

impl PlatformSurface {
    pub fn new() -> Result<Self> {
        let wayland = WaylandConnection::new().map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut event_loop = EventLoop::new().map_err(|e| anyhow::anyhow!("{}", e))?;

        if let Some(fd) = wayland.get_fd() {
            event_loop.register_wayland_fd(fd)?;
        }

        tracing::info!(
            display = wayland.display.as_str(),
            connected = wayland.is_connected(),
            "wayland connection initialized"
        );

        Ok(Self {
            wayland,
            event_loop,
            running: false,
        })
    }

    pub fn wayland_display(&self) -> &str {
        &self.wayland.display
    }

    pub fn is_wayland_connected(&self) -> bool {
        self.wayland.is_connected()
    }

    pub fn event_loop_mut(&mut self) -> &mut EventLoop {
        &mut self.event_loop
    }

    pub fn register_ipc_fd(
        &mut self,
        fd: std::os::fd::OwnedFd,
        interest: Interest,
    ) -> Result<Token> {
        self.event_loop
            .register_fd(fd, interest)
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    pub fn dispatch_loop(&mut self) -> Result<()> {
        struct DummyHandler;
        impl PlatformEventHandler for DummyHandler {
            fn handle_event(&self, event: PlatformEvent) -> bool {
                match event {
                    PlatformEvent::Wayland => {
                        tracing::debug!("wayland event received");
                    }
                    PlatformEvent::Ipc(fd) => {
                        tracing::debug!(fd = fd, "IPC event received");
                    }
                    PlatformEvent::Tick => {
                        tracing::trace!("tick event");
                    }
                }
                true
            }
        }

        self.running = true;
        tracing::info!(msg = "entering event loop, blocking on epoll");

        let handler = Arc::new(DummyHandler);
        self.event_loop
            .run(handler)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
        self.event_loop.stop();
    }
}

impl Default for PlatformSurface {
    fn default() -> Self {
        Self::new().expect("Failed to create platform surface")
    }
}
