//! Integration tests using real project schemas from schemas/ directory.

use schema_value::SchemaValue;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

fn load_schema(relative: &str) -> Option<Value> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(relative);
    if !path.exists() {
        eprintln!("schema not found, skipping: {}", path.display());
        return None;
    }
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    Some(serde_json::from_str(&content).expect("invalid JSON schema"))
}

// --- LinkState ---

#[test]
fn link_state_valid() {
    let Some(schema) = load_schema("schemas/gvt/link-state.schema.json") else { return; };
    let data = json!({
        "from": "node-us-1",
        "to": "node-jp-1",
        "latency": 150.5,
        "jitter": 12.3,
        "packet_loss": 0.01,
        "bandwidth": 1_000_000,
        "load": 0.45,
        "version": 42,
        "timestamp": 1700000000000_i64
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_ok());
}

#[test]
fn link_state_missing_required() {
    let Some(schema) = load_schema("schemas/gvt/link-state.schema.json") else { return; };
    // Missing "to" and "version" (required)
    let data = json!({
        "from": "node-1",
        "latency": 100,
        "jitter": 5,
        "timestamp": 1700000000000_i64
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_err());
}

#[test]
fn link_state_additional_property_rejected() {
    let Some(schema) = load_schema("schemas/gvt/link-state.schema.json") else { return; };
    let data = json!({
        "from": "a",
        "to": "b",
        "latency": 10,
        "jitter": 1,
        "version": 1,
        "timestamp": 1000,
        "extra_field": "not allowed"
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_err());
}

#[test]
fn link_state_pattern_validation() {
    let Some(schema) = load_schema("schemas/gvt/link-state.schema.json") else { return; };
    // "from" must match ^[a-zA-Z0-9_-]{1,64}$
    let data = json!({
        "from": "invalid node id with spaces!",
        "to": "ok-node",
        "latency": 10,
        "jitter": 1,
        "version": 1,
        "timestamp": 1000
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_err());
}

#[test]
fn link_state_range_violation() {
    let Some(schema) = load_schema("schemas/gvt/link-state.schema.json") else { return; };
    // packet_loss max is 1.0
    let data = json!({
        "from": "a",
        "to": "b",
        "latency": 10,
        "jitter": 1,
        "packet_loss": 1.5,
        "version": 1,
        "timestamp": 1000
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_err());
}

// --- NodeInfo ---

#[test]
fn node_info_valid() {
    let Some(schema) = load_schema("schemas/gvt/node-info.schema.json") else { return; };
    let data = json!({
        "id": "node-us-west-1",
        "type": "native",
        "region": "US",
        "addr": "192.168.1.1:4433",
        "capabilities": {
            "gvt_write": true,
            "routing": true,
            "exit": false
        },
        "capacity": 100,
        "current_load": 23,
        "version": 5,
        "timestamp": 1700000000000_i64
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_ok());
}

#[test]
fn node_info_invalid_enum() {
    let Some(schema) = load_schema("schemas/gvt/node-info.schema.json") else { return; };
    // "type" must be one of: native, clash, ss, vmess, trojan, relay
    let data = json!({
        "id": "node-1",
        "type": "invalid_type",
        "addr": "1.2.3.4:80",
        "capabilities": {"gvt_write": true, "routing": true, "exit": true},
        "version": 1,
        "timestamp": 1000
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_err());
}

#[test]
fn node_info_nested_capabilities_validation() {
    let Some(schema) = load_schema("schemas/gvt/node-info.schema.json") else { return; };
    // capabilities missing required "exit"
    let data = json!({
        "id": "node-1",
        "type": "native",
        "addr": "1.2.3.4:80",
        "capabilities": {"gvt_write": true, "routing": true},
        "version": 1,
        "timestamp": 1000
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_err());
}

#[test]
fn node_info_schema_propagation() {
    let Some(schema) = load_schema("schemas/gvt/node-info.schema.json") else { return; };
    let data = json!({
        "id": "node-1",
        "type": "native",
        "addr": "1.2.3.4:80",
        "capabilities": {"gvt_write": true, "routing": true, "exit": false},
        "version": 1,
        "timestamp": 1000
    });
    let sv = SchemaValue::new(data, schema);

    // get() propagates sub-schema
    let caps = sv.get("capabilities").unwrap();
    assert!(caps.schema().is_some());
    let cap_schema = caps.schema().unwrap();
    assert!(cap_schema.get("properties").is_some());

    // Further propagation to leaf
    let gvt_write = caps.get("gvt_write").unwrap();
    assert_eq!(gvt_write.to_value(), json!(true));
    assert!(gvt_write.schema().is_some());
}

// --- NodeConfig with defaults ---

#[test]
fn node_config_defaults() {
    let Some(schema) = load_schema("schemas/config/node-config.schema.json") else { return; };
    let data = json!({
        "listen": {},
        "cluster": {}
    });
    let sv = SchemaValue::new(data, schema).with_defaults();

    // listen defaults should be filled
    assert_eq!(sv.path("listen.proxy").unwrap().to_value(), json!("0.0.0.0:1080"));
    assert_eq!(sv.path("listen.quic").unwrap().to_value(), json!("0.0.0.0:4433"));
    assert_eq!(sv.path("listen.api").unwrap().to_value(), json!("127.0.0.1:8080"));
    assert_eq!(sv.path("listen.dns").unwrap().to_value(), json!(""));
}

#[test]
fn node_config_partial_defaults() {
    let Some(schema) = load_schema("schemas/config/node-config.schema.json") else { return; };
    let data = json!({
        "listen": {"proxy": "0.0.0.0:2080"},
        "cluster": {"mode": "active"}
    });
    let sv = SchemaValue::new(data, schema).with_defaults();

    // Explicit value kept
    assert_eq!(sv.path("listen.proxy").unwrap().to_value(), json!("0.0.0.0:2080"));
    // Other listen defaults filled
    assert_eq!(sv.path("listen.quic").unwrap().to_value(), json!("0.0.0.0:4433"));
    // Cluster mode kept
    assert_eq!(sv.path("cluster.mode").unwrap().to_value(), json!("active"));
    // Cluster defaults filled
    assert_eq!(sv.path("cluster.heartbeat_interval").unwrap().to_value(), json!(5000));
    assert_eq!(sv.path("cluster.timeout").unwrap().to_value(), json!(30000));
}

#[test]
fn node_config_valid_full() {
    let Some(schema) = load_schema("schemas/config/node-config.schema.json") else { return; };
    let data = json!({
        "node": {
            "id": "exit-us-1",
            "type": "native",
            "region": "US"
        },
        "listen": {
            "proxy": "0.0.0.0:1080",
            "quic": "0.0.0.0:4433",
            "api": "127.0.0.1:8080"
        },
        "cluster": {
            "seeds": ["10.0.0.1:4433", "10.0.0.2:4433"],
            "mode": "both",
            "heartbeat_interval": 5000,
            "timeout": 30000
        },
        "routing": {
            "profile": "stable",
            "top_k": 3
        },
        "rules_file": "rules.json"
    });
    let sv = SchemaValue::new(data, schema);
    assert!(sv.validate().is_ok());
}

#[test]
fn node_config_set_with_validation() {
    let Some(schema) = load_schema("schemas/config/node-config.schema.json") else { return; };
    let data = json!({
        "listen": {},
        "cluster": {}
    });
    let mut sv = SchemaValue::new(data, schema);

    // Set valid rules_file
    assert!(sv.set("rules_file", json!("custom-rules.json")).is_ok());
    assert_eq!(sv.get("rules_file").unwrap().to_value(), json!("custom-rules.json"));
}
