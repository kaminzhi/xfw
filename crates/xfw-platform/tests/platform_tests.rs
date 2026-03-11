use std::env;
use xfw_platform::PlatformSurface;

#[test]
fn test_platform_surface_new() {
    let surface = PlatformSurface::new();
    assert!(surface.is_ok());
}

#[test]
fn test_platform_surface_wayland_info() {
    let surface = PlatformSurface::new().unwrap();
    let _display = surface.wayland_display();
    let _connected = surface.is_wayland_connected();
}

#[test]
fn test_platform_surface_headless() {
    env::remove_var("WAYLAND_DISPLAY");
    let surface = PlatformSurface::new();
    assert!(surface.is_ok());
    let surface = surface.unwrap();
    assert!(!surface.is_wayland_connected());
}
