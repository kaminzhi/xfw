use std::collections::HashMap;
use std::os::fd::RawFd;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use mio::event::Event;
use mio::net::EventedFd;
use mio::{Events, Interest, Poll, PollOpt, Token};

use crate::error::PlatformError;
use crate::Result;

pub trait EventSource: Send + Sync {
    fn fd(&self) -> RawFd;
    fn ready(&self, event: &Event, dispatcher: &mut dyn EventDispatcher);
}

pub trait EventDispatcher: Send {
    fn dispatch(&mut self, source: &dyn EventSource, event: &Event);
}

pub struct EventLoop {
    poll: Poll,
    sources: HashMap<Token, Box<dyn EventSource>>,
    dispatchers: HashMap<Token, Box<dyn EventDispatcher>>,
    token_counter: usize,
    running: bool,
    wake_sender: Sender<()>,
    wake_receiver: Mutex<Receiver<()>>,
}

impl EventLoop {
    pub fn new() -> Result<Self> {
        let poll = Poll::new()
            .map_err(|e| PlatformError::EventLoop(format!("Failed to create poll: {}", e)))?;

        let (wake_sender, wake_receiver) = channel();

        Ok(Self {
            poll,
            sources: HashMap::new(),
            dispatchers: HashMap::new(),
            token_counter: 0,
            running: false,
            wake_sender,
            wake_receiver: Mutex::new(wake_receiver),
        })
    }

    pub fn register<S>(&mut self, source: S, dispatcher: impl EventDispatcher + 'static) -> Token
    where
        S: EventSource + 'static,
    {
        let token = Token(self.token_counter);
        self.token_counter += 1;

        let fd = source.fd();
        let evented = EventedFd::new(fd);

        self.poll
            .register(
                &evented,
                token,
                Interest::READABLE | Interest::WRITABLE,
                PollOpt::edge(),
            )
            .expect("Failed to register event source");

        self.sources.insert(token, Box::new(source));
        self.dispatchers.insert(token, Box::new(dispatcher));

        token
    }

    pub fn register_fd(&mut self, fd: RawFd, dispatcher: impl EventDispatcher + 'static) -> Token {
        let token = Token(self.token_counter);
        self.token_counter += 1;

        let evented = EventedFd::new(fd);

        self.poll
            .register(
                &evented,
                token,
                Interest::READABLE | Interest::WRITABLE,
                PollOpt::edge(),
            )
            .expect("Failed to register fd");

        let fd_source = FdEventSource { fd };
        self.sources.insert(token, Box::new(fd_source));
        self.dispatchers.insert(token, Box::new(dispatcher));

        token
    }

    pub fn unregister(&mut self, token: Token) {
        if let Some(source) = self.sources.remove(&token) {
            let fd = source.fd();
            let evented = EventedFd::new(fd);
            let _ = self.poll.unregister(&evented);
        }
        self.dispatchers.remove(&token);
    }

    pub fn wake(&self) {
        let _ = self.wake_sender.send(());
    }

    pub fn run<D>(&mut self, dispatcher: &mut D, timeout: Option<Duration>) -> Result<()>
    where
        D: EventDispatcher,
    {
        self.running = true;
        let mut events = Events::with_capacity(1024);

        while self.running {
            let wake_timeout = Duration::from_millis(100);

            let loop_timeout = timeout.unwrap_or(wake_timeout);

            self.poll
                .poll(&mut events, Some(loop_timeout))
                .map_err(|e| PlatformError::EventLoop(format!("Poll failed: {}", e)))?;

            if events.is_empty() {
                continue;
            }

            for event in events.iter() {
                let token = event.token();

                if token == Token(0) && (event.is_read_closed() || event.is_hup()) {
                    self.running = false;
                    break;
                }

                if let Some(source) = self.sources.get(&token) {
                    if let Some(d) = self.dispatchers.get_mut(&token) {
                        d.dispatch(source.as_ref(), &event);
                    }
                }
            }

            dispatcher.dispatch(
                &WakeEventSource,
                &Event::new(mio::Token(usize::MAX), mio::ready!(Ready::readable())),
            );

            if let Ok(receiver) = self.wake_receiver.lock() {
                while receiver.try_recv().is_ok() {
                    tracing::debug!("wake event received");
                }
            }
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running = false;
    }
}

use mio::Ready;

struct FdEventSource {
    fd: RawFd,
}

impl EventSource for FdEventSource {
    fn fd(&self) -> RawFd {
        self.fd
    }

    fn ready(&self, event: &Event, dispatcher: &mut dyn EventDispatcher) {
        dispatcher.dispatch(self, event);
    }
}

struct WakeEventSource;

impl EventSource for WakeEventSource {
    fn fd(&self) -> RawFd {
        std::hint::black_box(-1isize as RawFd)
    }

    fn ready(&self, event: &Event, dispatcher: &mut dyn EventDispatcher) {
        dispatcher.dispatch(self, event);
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new().expect("Failed to create event loop")
    }
}

pub struct Timer {
    deadline: Instant,
    duration: Duration,
    repeat: bool,
}

impl Timer {
    pub fn new(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            duration,
            repeat: false,
        }
    }

    pub fn repeating(duration: Duration) -> Self {
        Self {
            deadline: Instant::now() + duration,
            duration,
            repeat: true,
        }
    }

    pub fn is_ready(&self) -> bool {
        Instant::now() >= self.deadline
    }

    pub fn reset(&mut self) {
        if self.repeat {
            self.deadline = Instant::now() + self.duration;
        } else {
            self.deadline = Instant::now() + Duration::from_secs(u64::MAX);
        }
    }
}
