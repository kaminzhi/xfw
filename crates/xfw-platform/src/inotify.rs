use std::os::fd::AsRawFd;
use std::path::Path;

use inotify::{EventMask, Inotify, WatchMask};

use crate::error::PlatformResult;

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
    pub fn new() -> PlatformResult<Self> {
        let inotify =
            Inotify::init().map_err(|e| crate::error::PlatformError::FdError(e.to_string()))?;
        let fd = inotify.as_raw_fd();

        tracing::info!(fd = fd, "inotify initialized");

        Ok(Self { inotify, fd })
    }

    pub fn watch_file<P: AsRef<Path>>(&mut self, path: P) -> PlatformResult<()> {
        let path = path.as_ref();

        self.inotify
            .watches()
            .add(
                path,
                WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE | WatchMask::ACCESS,
            )
            .map_err(|e| {
                crate::error::PlatformError::FdError(format!(
                    "failed to watch {}: {}",
                    path.display(),
                    e
                ))
            })?;

        tracing::debug!(path = %path.display(), "watching file");
        Ok(())
    }

    pub fn watch_directory<P: AsRef<Path>>(&mut self, path: P) -> PlatformResult<()> {
        let path = path.as_ref();

        self.inotify
            .watches()
            .add(
                path,
                WatchMask::MODIFY | WatchMask::CREATE | WatchMask::DELETE,
            )
            .map_err(|e| {
                crate::error::PlatformError::FdError(format!(
                    "failed to watch dir {}: {}",
                    path.display(),
                    e
                ))
            })?;

        tracing::debug!(path = %path.display(), "watching directory");
        Ok(())
    }

    pub fn read_events(&mut self) -> PlatformResult<Vec<(String, FileEvent)>> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_file_watcher_new() {
        let watcher = FileWatcher::new();
        assert!(watcher.is_ok());
    }

    #[test]
    fn test_watch_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_file.txt");

        fs::write(&file_path, "initial").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        let result = watcher.watch_file(&file_path);
        assert!(result.is_ok());

        drop(temp_dir);
    }

    #[test]
    fn test_watch_directory() {
        let temp_dir = TempDir::new().unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        let result = watcher.watch_directory(temp_dir.path());
        assert!(result.is_ok());

        drop(temp_dir);
    }

    #[test]
    fn test_read_events() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        fs::write(&file_path, "initial").unwrap();

        let mut watcher = FileWatcher::new().unwrap();
        watcher.watch_file(&file_path).unwrap();

        fs::write(&file_path, "content").unwrap();

        std::thread::sleep(std::time::Duration::from_millis(50));

        let events = watcher.read_events().unwrap();
        assert!(!events.is_empty());

        drop(temp_dir);
    }
}
