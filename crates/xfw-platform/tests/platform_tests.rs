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
