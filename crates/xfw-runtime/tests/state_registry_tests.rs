use parking_lot::RwLock;
use std::sync::Arc;
use xfw_runtime::state::{NodeId, StateChange, StatePath, StateRegistry, StateSubscriber};

#[test]
fn test_state_path_matches() {
    let path = StatePath::new("global.battery");

    assert!(path.matches("global.battery"));
    assert!(path.matches("global.battery.level"));
    assert!(path.matches("global.battery.is_charging"));
    assert!(!path.matches("global.batteries"));
    assert!(!path.matches("other.path"));
}

#[test]
fn test_register_and_query() {
    let mut registry = StateRegistry::new();

    registry.register(StatePath::new("global.value"), NodeId::new(1));
    registry.register(StatePath::new("global.battery.level"), NodeId::new(2));
    registry.register(StatePath::new("global.battery.level"), NodeId::new(3));

    let affected = registry.get_affected_nodes("global.value");
    assert_eq!(affected.len(), 1);

    let affected = registry.get_affected_nodes("global.battery.level");
    assert_eq!(affected.len(), 2);
}

#[test]
fn test_unregister() {
    let mut registry = StateRegistry::new();

    registry.register(StatePath::new("global.value"), NodeId::new(1));
    registry.register(StatePath::new("global.value"), NodeId::new(2));

    registry.unregister_node(NodeId::new(1));

    let affected = registry.get_affected_nodes("global.value");
    assert_eq!(affected.len(), 1);
}

#[test]
fn test_subscriber_notification() {
    struct TestSubscriber {
        received: RwLock<Vec<StateChange>>,
    }

    impl StateSubscriber for TestSubscriber {
        fn on_state_change(&self, changes: &[StateChange]) {
            *self.received.write() = changes.to_vec();
        }
    }

    let subscriber = Arc::new(TestSubscriber {
        received: RwLock::new(Vec::new()),
    });
    let mut registry = StateRegistry::new();
    registry.add_subscriber(subscriber.clone());

    let changes = vec![StateChange {
        path: StatePath::new("global.value"),
        value: serde_json::json!(42),
    }];

    registry.notify_subscribers(&changes);

    let received = subscriber.received.read();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].path.as_str(), "global.value");
}
