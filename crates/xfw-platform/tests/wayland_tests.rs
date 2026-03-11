use xfw_platform::WaylandConnection;

#[test]
fn test_wayland_connection_new() {
    let conn = WaylandConnection::new();
    if conn.is_ok() {
        let conn = conn.unwrap();
        if conn.is_connected() {
            assert!(conn.get_fd().is_some());
        } else {
            tracing::warn!("No Wayland server running, test skipped");
        }
    }
}
