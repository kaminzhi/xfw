pub mod error;
pub mod event;
pub mod event_loop;
pub mod inotify;
pub mod platform;
pub mod wayland;

pub use error::{PlatformError, PlatformResult};
pub use event::{FdWatcher, PlatformEvent, PlatformEventHandler};
pub use event_loop::EventLoop;
pub use inotify::{FileEvent, FileWatcher};
pub use platform::PlatformSurface;
pub use wayland::WaylandConnection;
