use xfw_platform::WaylandConnection;

#[test]
fn test_wayland_connection_new() {
    let conn = WaylandConnection::new();
    if conn.is_ok() {
        let conn = conn.unwrap();
        let has_inner = conn.inner.is_some();
        if has_inner {
            assert!(conn.is_connected());
            assert!(conn.get_fd().is_some());
        } else {
            tracing::warn!("No Wayland server running, test skipped");
        }
    }
}
