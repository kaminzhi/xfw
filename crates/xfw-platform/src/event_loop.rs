use std::collections::HashMap;
use std::os::fd::IntoRawFd;
use std::sync::Arc;
use std::time::Duration;

use mio::event::Event;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Registry, Token};

use crate::error::{PlatformError, PlatformResult};
use crate::event::{FdWatcher, PlatformEvent, PlatformEventHandler};

pub struct EventLoop {
    poll: Poll,
    registry: Registry,
    events: Events,
    watchers: HashMap<Token, FdWatcher>,
    next_ipc_token: usize,
    wayland_fd: Option<i32>,
    running: bool,
}

impl EventLoop {
    pub fn new() -> PlatformResult<Self> {
        let poll = Poll::new().map_err(|e| PlatformError::EventLoop(e.to_string()))?;
        let registry = poll
            .registry()
            .try_clone()
            .map_err(|e| PlatformError::EventLoop(e.to_string()))?;
        Ok(Self {
            poll,
            registry,
            events: Events::with_capacity(1024),
            watchers: HashMap::new(),
            next_ipc_token: 1,
            wayland_fd: None,
            running: false,
        })
    }

    pub fn register_wayland_fd(&mut self, fd: i32) -> PlatformResult<()> {
        self.wayland_fd = Some(fd);
        let mut source = SourceFd(&fd);
        self.registry
            .register(
                &mut source,
                Token(0),
                Interest::READABLE | Interest::WRITABLE,
            )
            .map_err(|e: std::io::Error| PlatformError::EventLoop(e.to_string()))?;
        tracing::debug!(fd = fd, "registered wayland fd");
        Ok(())
    }

    pub fn register_fd<F>(&mut self, fd: F, interest: Interest) -> PlatformResult<Token>
    where
        F: IntoRawFd,
    {
        let fd = fd.into_raw_fd();
        let token = Token(self.next_ipc_token);
        self.next_ipc_token += 1;

        let mut source = SourceFd(&fd);
        self.registry
            .register(&mut source, token, interest)
            .map_err(|e: std::io::Error| PlatformError::EventLoop(e.to_string()))?;

        let watcher = FdWatcher::new(fd, token, interest);
        self.watchers.insert(token, watcher);

        tracing::debug!(fd = fd, token = ?token, "registered fd for IPC");
        Ok(token)
    }

    pub fn unregister_fd(&mut self, token: Token) -> PlatformResult<()> {
        if let Some(watcher) = self.watchers.remove(&token) {
            let mut source = SourceFd(&watcher.fd());
            self.registry
                .deregister(&mut source)
                .map_err(|e: std::io::Error| PlatformError::EventLoop(e.to_string()))?;
            tracing::debug!(token = ?token, "unregistered fd");
        }
        Ok(())
    }

    pub fn poll_events<F>(&mut self, timeout: Option<Duration>, handler: &F) -> PlatformResult<()>
    where
        F: PlatformEventHandler,
    {
        self.poll
            .poll(&mut self.events, timeout)
            .map_err(|e| PlatformError::EventLoop(e.to_string()))?;

        for event in &self.events {
            let platform_event = self.map_event(event);
            let should_continue = handler.handle_event(platform_event);
            if !should_continue {
                return Ok(());
            }
        }
        Ok(())
    }

    fn map_event(&self, event: &Event) -> PlatformEvent {
        if event.token() == Token(0) {
            return PlatformEvent::Wayland;
        }

        if let Some(watcher) = self.watchers.get(&event.token()) {
            if event.is_readable() {
                return PlatformEvent::Ipc(watcher.fd() as usize);
            }
        }

        PlatformEvent::Tick
    }

    pub fn run<H>(&mut self, handler: Arc<H>) -> PlatformResult<()>
    where
        H: PlatformEventHandler,
    {
        self.running = true;
        tracing::info!("starting event loop");

        while self.running {
            self.poll_events(Some(Duration::from_secs(3600)), &*handler)?;
        }

        tracing::info!("event loop stopped");
        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }

    pub fn is_running(&self) -> bool {
        self.running
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create event loop")
    }
}
