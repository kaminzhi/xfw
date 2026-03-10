use xfw_platform::clipboard::{Clipboard, ClipboardContent, DataDeviceHandler};

#[test]
fn test_clipboard_new() {
    let clipboard = Clipboard::new();
    assert!(clipboard.get_selection().is_none());
}

#[test]
fn test_clipboard_set_selection_text() {
    let mut clipboard = Clipboard::new();
    clipboard.set_selection(ClipboardContent::Text("Hello".to_string()));

    assert!(clipboard.get_selection().is_some());
    match clipboard.get_selection().unwrap() {
        ClipboardContent::Text(s) => assert_eq!(s, "Hello"),
        ClipboardContent::Image(_) => panic!("Expected text"),
    }
}

#[test]
fn test_clipboard_set_selection_image() {
    let mut clipboard = Clipboard::new();
    let image_data = vec![0u8; 100];
    clipboard.set_selection(ClipboardContent::Image(image_data.clone()));

    match clipboard.get_selection().unwrap() {
        ClipboardContent::Text(_) => panic!("Expected image"),
        ClipboardContent::Image(data) => assert_eq!(data.len(), 100),
    }
}

#[test]
fn test_clipboard_clear_selection() {
    let mut clipboard = Clipboard::new();
    clipboard.set_selection(ClipboardContent::Text("test".to_string()));
    assert!(clipboard.get_selection().is_some());

    clipboard.clear_selection();
    assert!(clipboard.get_selection().is_none());
}

#[test]
fn test_clipboard_offers() {
    let mut clipboard = Clipboard::new();

    // Add offers
    clipboard.add_offer(1, ClipboardContent::Text("offer1".to_string()));
    clipboard.add_offer(2, ClipboardContent::Text("offer2".to_string()));

    // Get offers
    assert!(clipboard.get_offer(1).is_some());
    assert!(clipboard.get_offer(2).is_some());
    assert!(clipboard.get_offer(3).is_none());

    // Remove offer
    clipboard.remove_offer(1);
    assert!(clipboard.get_offer(1).is_none());
}

#[test]
fn test_data_device_handler_new() {
    let handler = DataDeviceHandler::new();
    // Just verify it can be created
    let _ = handler;
}

#[test]
fn test_clipboard_content_variants() {
    let text = ClipboardContent::Text("test".to_string());
    let image = ClipboardContent::Image(vec![1, 2, 3]);

    // Verify both variants work
    match text {
        ClipboardContent::Text(s) => assert_eq!(s, "test"),
        _ => panic!("Expected text"),
    }

    match image {
        ClipboardContent::Image(data) => assert_eq!(data, vec![1, 2, 3]),
        _ => panic!("Expected image"),
    }
}
