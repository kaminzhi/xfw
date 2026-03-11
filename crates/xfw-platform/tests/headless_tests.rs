use std::env;

#[test]
fn test_headless_mode_when_no_wayland() {
    env::remove_var("WAYLAND_DISPLAY");

    let conn = xfw_platform::WaylandConnection::new();

    assert!(conn.is_ok());
    let conn = conn.unwrap();

    assert!(!conn.is_connected());
    assert!(conn.is_headless());
    assert!(conn.get_fd().is_none());
}

#[test]
fn test_display_name_always_set() {
    env::remove_var("WAYLAND_DISPLAY");

    let conn = xfw_platform::WaylandConnection::new().unwrap();

    assert_eq!(conn.display, "wayland-0");
}
