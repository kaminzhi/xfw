use xfw_platform::buffer::BufferConfig;

#[test]
fn test_buffer_config_default() {
    let config = BufferConfig::new(1920, 1080);
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
    assert_eq!(config.stride, 1920 * 4);
    // Format is XRGB8888 (value depends on wayland version)
    assert!(config.format > 0);
}

#[test]
fn test_buffer_config_with_stride() {
    let config = BufferConfig::new(800, 600).with_stride(3200);
    assert_eq!(config.stride, 3200);
}

#[test]
fn test_buffer_config_with_format() {
    let config = BufferConfig::new(800, 600).with_format(1);
    assert_eq!(config.format, 1);
}

#[test]
fn test_buffer_config_size() {
    let config = BufferConfig::new(1920, 1080);
    let expected_size = 1920 * 1080 * 4;
    assert_eq!(config.size(), expected_size);
}

#[test]
fn test_buffer_config_size_with_custom_stride() {
    let config = BufferConfig::new(1920, 1080).with_stride(3840);
    let expected_size = 3840 * 1080;
    assert_eq!(config.size(), expected_size);
}

// === Extreme Test Cases ===

#[test]
fn test_buffer_config_zero_size() {
    // Zero dimensions - should handle gracefully without panic
    let config = BufferConfig::new(0, 0);
    assert_eq!(config.width, 0);
    assert_eq!(config.height, 0);
    assert_eq!(config.stride, 0);
    // size() should return 0, not panic
    assert_eq!(config.size(), 0);
}

#[test]
fn test_buffer_config_extreme_size() {
    // Very large dimensions - check for potential overflow
    // Using moderate large values to avoid actual memory allocation
    let config = BufferConfig::new(16384, 16384);
    assert_eq!(config.width, 16384);
    assert_eq!(config.height, 16384);
    // stride = width * 4 = 65536 (fits in u32)
    assert_eq!(config.stride, 65536);
    // size = stride * height = 65536 * 16384 = 1073741824 (fits in usize, but may be 1GB)
    let size = config.size();
    assert!(size > 0);
    // Verify it doesn't cause integer overflow (would panic in debug)
    let expected_size = 65536usize * 16384usize;
    assert_eq!(size, expected_size);
}

#[test]
fn test_buffer_config_overflow_protection() {
    // Test moderate values that won't overflow
    let width: u32 = 16384; // 2^14
    let height: u32 = 16384; // 2^14

    let config = BufferConfig::new(width, height);

    // stride = width * 4 = 65536 (fits in u32)
    assert_eq!(config.stride, 65536);

    // size = stride * height = 65536 * 16384 = 1073741824 (1GB, fits in usize)
    let size = config.size();
    assert_eq!(size, 1073741824);
}
