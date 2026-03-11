use std::os::fd::{AsRawFd, RawFd};
use std::path::Path;

use inotify::{EventMask, Inotify, WatchMask};

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileEvent {
    Modified,
    Created,
    Deleted,
    Access,
    Other,
}

pub struct FileWatcher {
    inotify: Inotify,
    fd: i32,
}

impl FileWatcher {
    pub fn new() -> Result<Self> {
        let inotify = Inotify::init().map_err(|e| crate::error::buffer_error(e.to_string()))?;
        let fd = inotify.as_raw_fd();

        tracing::info!(fd = fd, "inotify initialized");

        Ok(Self { inotify, fd })
    }

    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        self.inotify
            .watches()
            .add(
                path,
                WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::ACCESS,
            )
            .map_err(|e| {
                crate::error::buffer_error(format!("failed to watch {}: {}", path.display(), e))
            })?;

        tracing::debug!(path = %path.display(), "watching file");
        Ok(())
    }

    pub fn watch_directory<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();

        self.inotify
            .watches()
            .add(
                path,
                WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE,
            )
            .map_err(|e| {
                crate::error::buffer_error(format!("failed to watch dir {}: {}", path.display(), e))
            })?;

        tracing::debug!(path = %path.display(), "watching directory");
        Ok(())
    }

    pub fn read_events(&mut self) -> Result<Vec<(String, FileEvent)>> {
        let mut events = Vec::new();
        let mut buffer = [0u8; 1024];

        match self.inotify.read_events(&mut buffer) {
            Ok(iter) => {
                for event in iter {
                    let path = event
                        .name
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let file_event = match event.mask {
                        m if m.contains(EventMask::MODIFY) => FileEvent::Modified,
                        m if m.contains(EventMask::CREATE) => FileEvent::Created,
                        m if m.contains(EventMask::DELETE) => FileEvent::Deleted,
                        m if m.contains(EventMask::ACCESS) => FileEvent::Access,
                        _ => FileEvent::Other,
                    };
                    events.push((path, file_event));
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "failed to read inotify events");
            }
        }

        Ok(events)
    }

    pub fn fd(&self) -> i32 {
        self.fd
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new().expect("Failed to create inotify watcher")
    }
}

impl AsRawFd for FileWatcher {
    fn as_raw_fd(&self) -> i32 {
        self.fd
    }
}
