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

// === Extreme Test Cases ===

#[test]
fn test_clipboard_empty_text() {
    // Empty string - should be valid
    let mut clipboard = Clipboard::new();
    clipboard.set_selection(ClipboardContent::Text("".to_string()));

    let result = clipboard.get_selection();
    assert!(result.is_some());
    match result.unwrap() {
        ClipboardContent::Text(s) => assert_eq!(s, ""),
        _ => panic!("Expected empty text"),
    }
}

#[test]
fn test_clipboard_very_long_text() {
    // Very long text - should not panic
    let long_text = "x".repeat(1_000_000); // 1MB string
    let mut clipboard = Clipboard::new();
    clipboard.set_selection(ClipboardContent::Text(long_text.clone()));

    let result = clipboard.get_selection();
    assert!(result.is_some());
    match result.unwrap() {
        ClipboardContent::Text(s) => assert_eq!(s.len(), 1_000_000),
        _ => panic!("Expected text"),
    }
}

#[test]
fn test_clipboard_huge_image() {
    // Large image data - verify it can be stored without immediate panic
    // (Actual memory pressure would occur at runtime, not in test)
    let huge_image = vec![0u8; 10 * 1024 * 1024]; // 10MB
    let mut clipboard = Clipboard::new();

    // This should not panic during creation
    clipboard.set_selection(ClipboardContent::Image(huge_image.clone()));

    let result = clipboard.get_selection();
    assert!(result.is_some());
    match result.unwrap() {
        ClipboardContent::Image(data) => assert_eq!(data.len(), 10 * 1024 * 1024),
        _ => panic!("Expected image"),
    }
}

#[test]
fn test_clipboard_binary_data() {
    // Binary data with null bytes - should be preserved
    let binary_data = vec![0u8, 1, 2, 0, 255, 0, 1, 2];
    let mut clipboard = Clipboard::new();
    clipboard.set_selection(ClipboardContent::Image(binary_data.clone()));

    let result = clipboard.get_selection();
    assert!(result.is_some());
    match result.unwrap() {
        ClipboardContent::Image(data) => {
            assert_eq!(data.len(), binary_data.len());
        }
        _ => panic!("Expected image"),
    }
}

#[test]
fn test_clipboard_multiple_offers() {
    // Multiple offers - should handle gracefully
    let mut clipboard = Clipboard::new();

    for i in 0..100 {
        clipboard.add_offer(i, ClipboardContent::Text(format!("offer{}", i)));
    }

    // All offers should be retrievable
    for i in 0..100 {
        assert!(clipboard.get_offer(i).is_some());
    }

    // Removing all should work
    for i in 0..100 {
        clipboard.remove_offer(i);
        assert!(clipboard.get_offer(i).is_none());
    }
}

#[test]
fn test_clipboard_clear_when_empty() {
    // Clear selection when already empty - should not panic
    let mut clipboard = Clipboard::new();

    // Clear multiple times
    clipboard.clear_selection();
    clipboard.clear_selection();
    clipboard.clear_selection();

    // Should still return None
    assert!(clipboard.get_selection().is_none());
}
