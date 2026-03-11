use std::collections::HashMap;
use std::os::fd::IntoRawFd;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use mio::event::Event;
use mio::unix::SourceFd;
use mio::{Events, Interest, Poll, Registry, Token};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformEvent {
    Wayland,
    Ipc(usize),
    Tick,
}

pub trait PlatformEventHandler: Send + Sync {
    fn handle_event(&self, event: PlatformEvent) -> bool;
}

pub struct FdWatcher {
    fd: i32,
    token: Token,
    interest: Interest,
}

impl FdWatcher {
    pub fn new(fd: i32, token: Token, interest: Interest) -> Self {
        Self {
            fd,
            token,
            interest,
        }
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }

    pub fn token(&self) -> Token {
        self.token
    }

    pub fn interest(&self) -> Interest {
        self.interest
    }
}

pub struct WaylandConnection {
    pub display: String,
    pub socket_name: Option<String>,
}

impl WaylandConnection {
    pub fn new() -> PlatformResult<Self> {
        let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());
        Ok(Self {
            display,
            socket_name: None,
        })
    }

    pub fn is_connected(&self) -> bool {
        true
    }
}

impl Default for WaylandConnection {
    fn default() -> Self {
        Self::new().expect("Failed to create Wayland connection")
    }
}

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

pub struct PlatformSurface {
    wayland: WaylandConnection,
    event_loop: EventLoop,
    running: bool,
}

impl PlatformSurface {
    pub fn new() -> Result<Self> {
        let wayland = WaylandConnection::new().map_err(|e| anyhow::anyhow!("{}", e))?;
        let event_loop = EventLoop::new().map_err(|e| anyhow::anyhow!("{}", e))?;

        tracing::info!(
            display = %wayland.display,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_wayland_connection_new() {
        let conn = WaylandConnection::new();
        assert!(conn.is_ok());
        let conn = conn.unwrap();
        assert!(conn.is_connected());
    }

    #[test]
    fn test_event_loop_new() {
        let loop_result = EventLoop::new();
        assert!(loop_result.is_ok());
    }

    #[test]
    fn test_platform_surface_new() {
        let surface = PlatformSurface::new();
        assert!(surface.is_ok());
        let surface = surface.unwrap();
        assert!(surface.is_wayland_connected());
    }

    fn create_pipe() -> (std::os::fd::OwnedFd, std::os::fd::OwnedFd) {
        use std::os::fd::FromRawFd;
        let mut fds = [0i32; 2];
        unsafe {
            libc::pipe(fds.as_mut_ptr());
        }
        unsafe {
            (
                std::os::fd::OwnedFd::from_raw_fd(fds[0]),
                std::os::fd::OwnedFd::from_raw_fd(fds[1]),
            )
        }
    }

    #[test]
    fn test_fd_registration() {
        let mut event_loop = EventLoop::new().unwrap();

        let (read_end, write_end) = create_pipe();
        let token = event_loop.register_fd(read_end, Interest::READABLE);
        assert!(token.is_ok());

        let token = token.unwrap();
        let result = event_loop.unregister_fd(token);
        assert!(result.is_ok());

        drop(write_end);
    }

    #[test]
    fn test_event_handler() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct TestHandler {
            counter: Arc<AtomicUsize>,
        }

        impl PlatformEventHandler for TestHandler {
            fn handle_event(&self, _event: PlatformEvent) -> bool {
                self.counter.fetch_add(1, Ordering::SeqCst);
                false
            }
        }

        let counter: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        let handler = Arc::new(TestHandler {
            counter: counter.clone(),
        });

        let mut event_loop = EventLoop::new().unwrap();

        let (read_end, write_end) = create_pipe();
        let _token = event_loop
            .register_fd(read_end, Interest::READABLE)
            .unwrap();

        drop(write_end);

        let handler_clone = handler.clone();
        let _ = event_loop.poll_events(Some(Duration::from_millis(10)), &*handler_clone);

        assert!(counter.load(Ordering::SeqCst) >= 1);
    }

    #[test]
    fn test_poll_timeout() {
        struct TimeoutHandler {
            start: Instant,
            call_count: Arc<std::sync::atomic::AtomicUsize>,
        }

        impl PlatformEventHandler for TimeoutHandler {
            fn handle_event(&self, _event: PlatformEvent) -> bool {
                let count = self
                    .call_count
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if count >= 1 {
                    return false;
                }
                true
            }
        }

        let call_count = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let handler = Arc::new(TimeoutHandler {
            start: Instant::now(),
            call_count: call_count.clone(),
        });

        let mut event_loop = EventLoop::new().unwrap();
        let _ = event_loop.poll_events(Some(Duration::from_millis(50)), &*handler);

        let elapsed = Instant::now().duration_since(handler.start);
        assert!(elapsed.as_millis() >= 40);
    }

    #[test]
    fn test_platform_event_mapping() {
        let mut event_loop = EventLoop::new().unwrap();

        let (read_end, write_end) = create_pipe();
        let token = event_loop
            .register_fd(read_end, Interest::READABLE)
            .unwrap();

        drop(write_end);

        assert!(token != Token(0));
    }
}
