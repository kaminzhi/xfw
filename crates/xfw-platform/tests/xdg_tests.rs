use xfw_platform::xdg::{WindowResizeEdge, XdgWindowConfig, XdgWindowManager};

#[test]
fn test_xdg_window_config_default() {
    let config = XdgWindowConfig::default();
    assert_eq!(config.title, "xfw");
    assert_eq!(config.app_id, Some("xfw".to_string()));
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert_eq!(config.min_width, 0);
    assert_eq!(config.min_height, 0);
    assert_eq!(config.max_width, 0);
    assert_eq!(config.max_height, 0);
    assert!(config.decorations);
    assert!(config.resizable);
    assert!(!config.fullscreen);
    assert!(!config.maximized);
    assert!(!config.minimized);
    assert!(config.focus);
}

#[test]
fn test_xdg_window_config_new() {
    let config = XdgWindowConfig::new("Test Window", 640, 480);
    assert_eq!(config.title, "Test Window");
    assert_eq!(config.width, 640);
    assert_eq!(config.height, 480);
}

#[test]
fn test_xdg_window_config_builder() {
    let config = XdgWindowConfig::new("Custom", 1024, 768)
        .with_app_id("com.test.app")
        .with_min_size(320, 240)
        .with_max_size(1920, 1080)
        .with_decorations(false)
        .with_resizable(false)
        .fullscreen(true)
        .maximized(true);

    assert_eq!(config.title, "Custom");
    assert_eq!(config.app_id, Some("com.test.app".to_string()));
    assert_eq!(config.min_width, 320);
    assert_eq!(config.min_height, 240);
    assert_eq!(config.max_width, 1920);
    assert_eq!(config.max_height, 1080);
    assert!(!config.decorations);
    assert!(!config.resizable);
    assert!(config.fullscreen);
    assert!(config.maximized);
}

#[test]
fn test_window_resize_edge_variants() {
    // Just verify the enum variants exist
    let _none = WindowResizeEdge::None;
    let _top = WindowResizeEdge::Top;
    let _bottom = WindowResizeEdge::Bottom;
    let _left = WindowResizeEdge::Left;
    let _right = WindowResizeEdge::Right;
    let _top_left = WindowResizeEdge::TopLeft;
    let _top_right = WindowResizeEdge::TopRight;
    let _bottom_left = WindowResizeEdge::BottomLeft;
    let _bottom_right = WindowResizeEdge::BottomRight;
}

#[test]
fn test_xdg_window_manager_new() {
    let manager = XdgWindowManager::new();
    assert_eq!(manager.windows().count(), 0);
    assert!(manager.get_focused().is_none());
}

#[test]
fn test_xdg_window_manager_focused() {
    let mut manager = XdgWindowManager::new();

    // Initially no focus
    assert!(manager.get_focused().is_none());

    // Set focus
    manager.set_focused(1);
    assert_eq!(manager.get_focused(), Some(1));

    // Change focus
    manager.set_focused(2);
    assert_eq!(manager.get_focused(), Some(2));
}
