use xfw_platform::surface::{
    Anchor, KeyboardInteractivity, Layer, LayerSurfaceConfig, SurfaceManager,
};

#[test]
fn test_anchor_variants() {
    // Just verify the enum variants exist and can be created
    let _top = Anchor::Top;
    let _bottom = Anchor::Bottom;
    let _left = Anchor::Left;
    let _right = Anchor::Right;
    let _top_left = Anchor::TopLeft;
    let _top_right = Anchor::TopRight;
    let _bottom_left = Anchor::BottomLeft;
    let _bottom_right = Anchor::BottomRight;
    let _all = Anchor::All;
}

#[test]
fn test_layer_surface_config_default() {
    let config = LayerSurfaceConfig::default();
    assert_eq!(config.anchor, Anchor::Top);
    assert_eq!(config.layer, Layer::Top);
    assert_eq!(config.keyboard_interactivity, KeyboardInteractivity::None);
    assert_eq!(config.margin, (0, 0, 0, 0));
    assert_eq!(config.namespace, "xfw");
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
}

#[test]
fn test_layer_surface_config_custom() {
    let config = LayerSurfaceConfig {
        anchor: Anchor::Bottom,
        layer: Layer::Overlay,
        keyboard_interactivity: KeyboardInteractivity::OnDemand,
        margin: (10, 20, 30, 40),
        namespace: "test".to_string(),
        width: 800,
        height: 100,
    };
    assert_eq!(config.anchor, Anchor::Bottom);
    assert_eq!(config.layer, Layer::Overlay);
    assert_eq!(
        config.keyboard_interactivity,
        KeyboardInteractivity::OnDemand
    );
    assert_eq!(config.margin, (10, 20, 30, 40));
    assert_eq!(config.namespace, "test");
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 100);
}

#[test]
fn test_surface_manager_new() {
    let manager = SurfaceManager::new();
    assert_eq!(manager.surfaces().count(), 0);
}

#[test]
fn test_surface_manager_add_get_remove() {
    // SurfaceManager can't be tested fully without Wayland connection
    // but we can verify the API structure
    let mut manager = SurfaceManager::new();

    // Verify we can create and manage the manager
    assert!(manager.get_surface(0).is_none());
    assert!(manager.get_surface_mut(0).is_none());

    let removed = manager.remove_surface(0);
    assert!(removed.is_none());
}

// === Extreme Test Cases ===

#[test]
fn test_layer_config_zero_size() {
    // Zero size - should be handled
    let config = LayerSurfaceConfig {
        anchor: Anchor::Top,
        layer: Layer::Top,
        keyboard_interactivity: KeyboardInteractivity::None,
        margin: (0, 0, 0, 0),
        namespace: "test".to_string(),
        width: 0,
        height: 0,
    };
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
}

#[test]
fn test_layer_config_extreme_size() {
    // Very large dimensions
    let config = LayerSurfaceConfig {
        anchor: Anchor::All,
        layer: Layer::Overlay,
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        margin: (1000, 2000, 3000, 4000),
        namespace: "large".to_string(),
        width: 16384,
        height: 16384,
    };
    assert_eq!(config.width, 16384);
    assert_eq!(config.height, 16384);
    assert_eq!(config.margin, (1000, 2000, 3000, 4000));
}

#[test]
fn test_layer_config_negative_margin() {
    // Negative margins (allowed by protocol)
    let config = LayerSurfaceConfig {
        anchor: Anchor::Top,
        layer: Layer::Top,
        keyboard_interactivity: KeyboardInteractivity::None,
        margin: (-10, -20, -30, -40),
        namespace: "test".to_string(),
        width: 100,
        height: 50,
    };
    assert_eq!(config.margin, (-10, -20, -30, -40));
}

#[test]
fn test_layer_config_very_long_namespace() {
    // Very long namespace
    let long_ns = "x".repeat(10000);
    let config = LayerSurfaceConfig {
        anchor: Anchor::Top,
        layer: Layer::Top,
        keyboard_interactivity: KeyboardInteractivity::None,
        margin: (0, 0, 0, 0),
        namespace: long_ns,
        width: 100,
        height: 50,
    };
    assert_eq!(config.namespace.len(), 10000);
}

#[test]
fn test_layer_variants() {
    // Verify all layer variants
    let _bg = Layer::Background;
    let _bottom = Layer::Bottom;
    let _top = Layer::Top;
    let _overlay = Layer::Overlay;
}

#[test]
fn test_keyboard_interactivity_variants() {
    // Verify all keyboard interactivity variants
    let _none = KeyboardInteractivity::None;
    let _exclusive = KeyboardInteractivity::Exclusive;
    let _ondemand = KeyboardInteractivity::OnDemand;
}

#[test]
fn test_surface_manager_many_surfaces() {
    // Many surfaces
    let mut manager = SurfaceManager::new();

    // Note: We can't actually add surfaces without a Wayland connection
    // But we can verify the manager handles the API calls
    for i in 0..1000 {
        assert!(manager.get_surface(i).is_none());
        assert!(manager.get_surface_mut(i).is_none());
        let _ = manager.remove_surface(i);
    }
}

#[test]
fn test_surface_manager_zero_id() {
    // Zero is a valid surface ID
    let mut manager = SurfaceManager::new();

    assert!(manager.get_surface(0).is_none());
    assert!(manager.get_surface_mut(0).is_none());
}
