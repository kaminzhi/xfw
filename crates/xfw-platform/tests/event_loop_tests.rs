use std::time::{Duration, Instant};
use xfw_platform::event_loop::Timer;
use xfw_platform::surface::{Anchor, KeyboardInteractivity, Layer, LayerSurfaceConfig};
use xfw_platform::{PlatformEvent, SurfaceGeometry};

#[test]
fn test_timer_new() {
    let timer = Timer::new(Duration::from_secs(1));
    assert!(!timer.is_ready());
}

#[test]
fn test_timer_repeating() {
    let mut timer = Timer::repeating(Duration::from_millis(10));
    timer.reset();
    let _ = timer.is_ready();
}

#[test]
fn test_timer_non_repeating() {
    let mut timer = Timer::new(Duration::from_millis(10));
    timer.reset();
    let before = Instant::now();
    let _ = timer.is_ready();
    let elapsed = before.elapsed();
    assert!(elapsed < Duration::from_millis(5));
}

#[test]
fn test_timer_duration() {
    let duration = Duration::from_secs(2);
    let timer = Timer::new(duration);
    let _ = timer.is_ready();
}

#[test]
fn test_platform_event_variants() {
    let _event = PlatformEvent::Wake;
    let _event = PlatformEvent::Quit;
    let _event = PlatformEvent::PointerEnter {
        surface_id: 1,
        x: 10.0,
        y: 20.0,
    };
    let _event = PlatformEvent::PointerLeave { surface_id: 1 };
    let _event = PlatformEvent::PointerMove {
        surface_id: 1,
        x: 10.0,
        y: 20.0,
    };
    let _event = PlatformEvent::PointerButton {
        surface_id: 1,
        button: 1,
        pressed: true,
    };
    let _event = PlatformEvent::Keyboard {
        surface_id: 1,
        key: 50,
        pressed: true,
    };
    let _event = PlatformEvent::ConfigChanged {
        surface_id: 1,
        width: 800,
        height: 600,
    };
    let _event = PlatformEvent::Keymap { fd: 0, size: 1024 };
    let _event = PlatformEvent::DataReceived {
        surface_id: 1,
        data: vec![1, 2, 3],
    };
}

#[test]
fn test_surface_geometry() {
    let geo = SurfaceGeometry {
        x: 10.0,
        y: 20.0,
        width: 800,
        height: 600,
    };
    assert_eq!(geo.x, 10.0);
    assert_eq!(geo.y, 20.0);
    assert_eq!(geo.width, 800);
    assert_eq!(geo.height, 600);
}

#[test]
fn test_surface_geometry_clone() {
    let geo = SurfaceGeometry {
        x: 10.0,
        y: 20.0,
        width: 800,
        height: 600,
    };
    let geo_clone = geo.clone();
    assert_eq!(geo.x, geo_clone.x);
    assert_eq!(geo.y, geo_clone.y);
    assert_eq!(geo.width, geo_clone.width);
    assert_eq!(geo.height, geo_clone.height);
}

#[test]
fn test_layer_surface_config_defaults() {
    let config = LayerSurfaceConfig::default();
    assert_eq!(config.anchor, Anchor::Top);
    assert_eq!(config.layer, Layer::Top);
    assert_eq!(config.keyboard_interactivity, KeyboardInteractivity::None);
    assert_eq!(config.namespace, "xfw");
}

#[test]
fn test_layer_surface_config_custom() {
    let config = LayerSurfaceConfig {
        anchor: Anchor::Bottom,
        layer: Layer::Overlay,
        keyboard_interactivity: KeyboardInteractivity::Exclusive,
        margin: (10, 20, 30, 40),
        namespace: "custom".to_string(),
        width: 800,
        height: 100,
    };
    assert_eq!(config.anchor, Anchor::Bottom);
    assert_eq!(config.layer, Layer::Overlay);
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 100);
}

#[test]
fn test_platform_event_debug() {
    let event = PlatformEvent::Wake;
    let debug_str = format!("{:?}", event);
    assert!(debug_str.contains("Wake"));
}
