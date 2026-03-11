use mio::{Interest, Token};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use xfw_platform::{EventLoop, FdWatcher, PlatformEvent, PlatformEventHandler};

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
fn test_event_loop_new() {
    let loop_result = EventLoop::new();
    assert!(loop_result.is_ok());
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
    struct TestHandler {
        counter: std::sync::Arc<AtomicUsize>,
    }

    impl PlatformEventHandler for TestHandler {
        fn handle_event(&self, _event: PlatformEvent) -> bool {
            self.counter.fetch_add(1, Ordering::SeqCst);
            false
        }
    }

    let counter: std::sync::Arc<AtomicUsize> = std::sync::Arc::new(AtomicUsize::new(0));
    let handler = std::sync::Arc::new(TestHandler {
        counter: counter.clone(),
    });

    let mut event_loop = EventLoop::new().unwrap();

    let (read_end, write_end) = create_pipe();
    let _token = event_loop
        .register_fd(read_end, Interest::READABLE)
        .unwrap();

    drop(write_end);

    let handler_clone = handler.clone();
    let _ = event_loop.poll_events(Some(std::time::Duration::from_millis(10)), &*handler_clone);

    assert!(counter.load(Ordering::SeqCst) >= 1);
}

#[test]
fn test_poll_timeout() {
    struct TimeoutHandler {
        start: Instant,
        call_count: std::sync::Arc<AtomicUsize>,
    }

    impl PlatformEventHandler for TimeoutHandler {
        fn handle_event(&self, _event: PlatformEvent) -> bool {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst);
            if count >= 1 {
                return false;
            }
            true
        }
    }

    let call_count = std::sync::Arc::new(AtomicUsize::new(0));
    let handler = std::sync::Arc::new(TimeoutHandler {
        start: Instant::now(),
        call_count: call_count.clone(),
    });

    let mut event_loop = EventLoop::new().unwrap();
    let _ = event_loop.poll_events(Some(std::time::Duration::from_millis(50)), &*handler);

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

#[test]
fn test_fd_watcher() {
    let watcher = FdWatcher::new(42, Token(1), Interest::READABLE);
    assert_eq!(watcher.fd(), 42);
    assert_eq!(watcher.token(), Token(1));
    assert_eq!(watcher.interest(), Interest::READABLE);
}

#[test]
fn test_event_loop_stop() {
    let mut event_loop = EventLoop::new().unwrap();
    assert!(!event_loop.is_running());
    event_loop.stop();
    assert!(!event_loop.is_running());
}

#[test]
fn test_event_loop_is_running() {
    let mut event_loop = EventLoop::new().unwrap();
    assert!(!event_loop.is_running());
    event_loop.stop();
    assert!(!event_loop.is_running());
}
