use xfw_platform::surface::{
    Anchor, KeyboardInteractivity, Layer, LayerSurfaceConfig, SurfaceManager,
};

#[test]
fn test_anchor_to_wl() {
    assert_eq!(Anchor::Top.to_wl(), 1);
    assert_eq!(Anchor::Bottom.to_wl(), 2);
    assert_eq!(Anchor::Left.to_wl(), 4);
    assert_eq!(Anchor::Right.to_wl(), 8);
}

#[test]
fn test_anchor_to_wl_combined() {
    let top_left = Anchor::TopLeft.to_wl();
    assert!(top_left & 1 != 0); // Top
    assert!(top_left & 4 != 0); // Left

    let bottom_right = Anchor::BottomRight.to_wl();
    assert!(bottom_right & 2 != 0); // Bottom
    assert!(bottom_right & 8 != 0); // Right

    let all = Anchor::All.to_wl();
    assert_eq!(all, 15); // 1 | 2 | 4 | 8 = 15
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
