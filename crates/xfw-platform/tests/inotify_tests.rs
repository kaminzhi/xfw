use std::fs;
use tempfile::TempDir;
use xfw_platform::{FileEvent, FileWatcher};

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

#[test]
fn test_file_event_types() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    fs::write(&file_path, "initial").unwrap();

    let mut watcher = FileWatcher::new().unwrap();
    watcher.watch_file(&file_path).unwrap();

    fs::write(&file_path, "modified").unwrap();

    std::thread::sleep(std::time::Duration::from_millis(50));

    let events = watcher.read_events().unwrap();
    assert!(!events.is_empty());

    let (_, event) = &events[0];
    assert!(matches!(event, FileEvent::Modified));

    drop(temp_dir);
}

#[test]
fn test_file_watcher_fd() {
    let watcher = FileWatcher::new().unwrap();
    let fd = watcher.fd();
    assert!(fd > 0);
}
