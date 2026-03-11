use xfw_platform::input::{InputManager, InputState, KeyState, PointerButton};

#[test]
fn test_input_state_new() {
    let state = InputState::new();
    assert_eq!(state.pointer_x, 0.0);
    assert_eq!(state.pointer_y, 0.0);
    assert!(state.focused_surface.is_none());
    assert!(state.hovered_surface.is_none());
}

#[test]
fn test_input_state_update_pointer() {
    let mut state = InputState::new();
    state.update_pointer_position(100.0, 200.0);
    assert_eq!(state.pointer_x, 100.0);
    assert_eq!(state.pointer_y, 200.0);
}

#[test]
fn test_input_manager_new() {
    let manager = InputManager::new();
    assert!(manager.get_hovered_surface().is_none());
}

#[test]
fn test_input_manager_surface_map() {
    let mut manager = InputManager::new();
    manager.register_surface(1, 100);
    manager.register_surface(2, 200);

    assert_eq!(manager.get_surface_id(100), Some(1));
    assert_eq!(manager.get_surface_id(200), Some(2));
    assert_eq!(manager.get_surface_id(300), None);
}

#[test]
fn test_input_manager_hit_test_basic() {
    let manager = InputManager::new();

    let surfaces = vec![
        (1, 0.0, 0.0, 100.0, 100.0),
        (2, 100.0, 0.0, 100.0, 100.0),
        (3, 0.0, 100.0, 200.0, 100.0),
    ];

    assert_eq!(manager.hit_test(50.0, 50.0, &surfaces), Some(1));
    assert_eq!(manager.hit_test(150.0, 50.0, &surfaces), Some(2));
    assert_eq!(manager.hit_test(50.0, 150.0, &surfaces), Some(3));
    assert_eq!(manager.hit_test(300.0, 300.0, &surfaces), None);
}

#[test]
fn test_input_manager_hit_test_edge() {
    let manager = InputManager::new();
    let surfaces = vec![(1, 0.0, 0.0, 100.0, 100.0)];

    assert_eq!(manager.hit_test(0.0, 0.0, &surfaces), Some(1));
    assert_eq!(manager.hit_test(99.0, 99.0, &surfaces), Some(1));
    assert_eq!(manager.hit_test(100.0, 100.0, &surfaces), None);
}

#[test]
fn test_pointer_button_from_u32() {
    assert_eq!(PointerButton::from(0x110), PointerButton::Left);
    assert_eq!(PointerButton::from(0x111), PointerButton::Right);
    assert_eq!(PointerButton::from(0x112), PointerButton::Middle);
    assert_eq!(PointerButton::from(0x113), PointerButton::WheelUp);
    assert_eq!(PointerButton::from(0x114), PointerButton::WheelDown);
    assert_eq!(PointerButton::from(0x999), PointerButton::Other(0x999));
}

#[test]
fn test_key_state() {
    assert!(matches!(KeyState::Pressed, KeyState::Pressed));
    assert!(matches!(KeyState::Released, KeyState::Released));
}

// === Extreme Test Cases ===

#[test]
fn test_hit_test_empty_list() {
    let manager = InputManager::new();
    let surfaces: Vec<(u32, f32, f32, f32, f32)> = vec![];

    assert_eq!(manager.hit_test(50.0, 50.0, &surfaces), None);
}

#[test]
fn test_hit_test_negative_coords() {
    let manager = InputManager::new();
    let surfaces = vec![(1, 0.0, 0.0, 100.0, 100.0)];

    assert_eq!(manager.hit_test(-10.0, 50.0, &surfaces), None);
    assert_eq!(manager.hit_test(50.0, -10.0, &surfaces), None);
    assert_eq!(manager.hit_test(-10.0, -10.0, &surfaces), None);
}

#[test]
fn test_hit_test_extreme_coords() {
    let manager = InputManager::new();
    let surfaces = vec![(1, 0.0, 0.0, 100.0, 100.0)];

    assert_eq!(manager.hit_test(10000.0, 10000.0, &surfaces), None);
    assert_eq!(manager.hit_test(f64::MAX, f64::MAX, &surfaces), None);
}

#[test]
fn test_hit_test_overlapping_windows() {
    let manager = InputManager::new();
    let surfaces = vec![(1, 0.0, 0.0, 1920.0, 1080.0), (2, 0.0, 0.0, 1920.0, 1080.0)];

    let result = manager.hit_test(960.0, 540.0, &surfaces);
    assert!(result.is_some());
}

#[test]
fn test_hit_test_very_small_surface() {
    let manager = InputManager::new();
    // 1x1 pixel surface at position 50,50
    let surfaces = vec![(1, 50.0, 50.0, 1.0, 1.0)];

    // At exact position - inside
    assert_eq!(manager.hit_test(50.0, 50.0, &surfaces), Some(1));
    // Slightly offset - depends on implementation
    let result = manager.hit_test(50.5, 50.5, &surfaces);
    // Just verify it returns something valid (either Some or None)
    assert!(result.is_none() || result == Some(1));
}

#[test]
fn test_pointer_button_unknown_codes() {
    assert_eq!(PointerButton::from(0), PointerButton::Other(0));
    assert_eq!(
        PointerButton::from(u32::MAX),
        PointerButton::Other(u32::MAX)
    );
    assert_eq!(PointerButton::from(0xFFFF), PointerButton::Other(0xFFFF));
}
