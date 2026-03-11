use mio::Interest;

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
    token: mio::Token,
    interest: Interest,
}

impl FdWatcher {
    pub fn new(fd: i32, token: mio::Token, interest: Interest) -> Self {
        Self {
            fd,
            token,
            interest,
        }
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }

    pub fn token(&self) -> mio::Token {
        self.token
    }

    pub fn interest(&self) -> Interest {
        self.interest
    }
}
