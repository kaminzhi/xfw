use xfw_platform::input::{InputManager, InputState, KeyState, PointerAxis, PointerButton};

#[test]
fn test_input_state_new() {
    let state = InputState::new();
    assert_eq!(state.pointer_x, 0.0);
    assert_eq!(state.pointer_y, 0.0);
    assert!(state.focused_surface.is_none());
    assert!(state.hovered_surface.is_none());
}

#[test]
fn test_input_state_update_pointer_position() {
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
fn test_input_manager_surface_mapping() {
    let mut manager = InputManager::new();
    manager.register_surface(1, 100);
    manager.register_surface(2, 200);

    assert_eq!(manager.get_surface_id(100), Some(1));
    assert_eq!(manager.get_surface_id(200), Some(2));
    assert_eq!(manager.get_surface_id(300), None);
}

#[test]
fn test_input_manager_hit_test() {
    let manager = InputManager::new();

    // Surfaces: (id, x, y, width, height)
    let surfaces = vec![
        (1, 0.0, 0.0, 100.0, 100.0),   // surface 1
        (2, 100.0, 0.0, 100.0, 100.0), // surface 2
        (3, 0.0, 100.0, 200.0, 100.0), // surface 3 (overlaps)
    ];

    // Inside surface 1
    assert_eq!(manager.hit_test(50.0, 50.0, &surfaces), Some(1));

    // Inside surface 2
    assert_eq!(manager.hit_test(150.0, 50.0, &surfaces), Some(2));

    // Inside surface 3
    assert_eq!(manager.hit_test(50.0, 150.0, &surfaces), Some(3));

    // Outside all surfaces
    assert_eq!(manager.hit_test(300.0, 300.0, &surfaces), None);
}

#[test]
fn test_input_manager_hit_test_edge_cases() {
    let manager = InputManager::new();
    let surfaces = vec![(1, 0.0, 0.0, 100.0, 100.0)];

    // On edge (should be inside based on implementation)
    assert_eq!(manager.hit_test(0.0, 0.0, &surfaces), Some(1));
    assert_eq!(manager.hit_test(99.0, 99.0, &surfaces), Some(1));

    // Just outside
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
