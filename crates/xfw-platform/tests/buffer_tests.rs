use xfw_platform::buffer::BufferConfig;

#[test]
fn test_buffer_config_default() {
    let config = BufferConfig::new(1920, 1080);
    assert_eq!(config.width, 1920);
    assert_eq!(config.height, 1080);
    assert_eq!(config.stride, 1920 * 4);
    assert_eq!(config.format, 2); // XRGB8888
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
