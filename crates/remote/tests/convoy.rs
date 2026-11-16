use serde_json::json;
use std::time::{Duration, Instant};
use westplay_remote::convoy::Convoy;
fn frame() -> serde_json::Value {
    json!({"status":"candidate-sample","frame":1,"readMs":3.0,"vehicles":[{"id":"aaaaaaaaaaaa","x":100.0,"z":200.0},{"id":"bbbbbbbbbbbb","x":130.0,"z":240.0}],"ownTelemetry":{"x":100.0,"z":200.0}})
}
#[test]
fn self_is_removed_and_only_relative_coordinates_reach_web() {
    let now = Instant::now();
    let mut c = Convoy::default();
    c.accept(&frame(), now);
    let v = c.snapshot(now);
    assert_eq!(v["players"].as_array().unwrap().len(), 1);
    assert_eq!(v["players"][0]["distance"], 50.0);
    assert_eq!(v["players"][0]["dx"], 30.0);
    assert!(v["players"][0].get("x").is_none());
    assert!(!v.to_string().contains("aaaaaaaaaaaa"));
}
#[test]
fn missing_own_ambiguous_or_expired_data_never_holds_old_markers() {
    let now = Instant::now();
    let mut c = Convoy::default();
    c.accept(&frame(), now);
    assert!(c.snapshot(now + Duration::from_millis(2001))["players"].as_array().unwrap().is_empty());
    let mut f = frame();
    f["ownTelemetry"] = serde_json::Value::Null;
    c.accept(&f, now);
    assert!(c.snapshot(now)["players"].as_array().unwrap().is_empty());
    let mut f = frame();
    f["vehicles"][1]["x"] = json!(100.0);
    f["vehicles"][1]["z"] = json!(200.0);
    c.accept(&f, now);
    assert!(c.snapshot(now)["players"].as_array().unwrap().is_empty());
}
#[test]
fn logs_are_bounded_and_never_include_payload_credentials_or_absolute_positions() {
    let now = Instant::now();
    let mut c = Convoy::default();
    for _ in 0..600 {
        c.accept(&frame(), now);
    }
    let v = c.logs();
    assert!(v.len() <= 256);
    let s = serde_json::to_string(&v).unwrap();
    assert!(!s.contains("aaaaaaaaaaaa"));
    assert!(!s.contains("ownTelemetry"));
    assert!(!s.contains("200.0"));
}

#[test]
fn native_names_are_bound_sanitized_and_do_not_leak_into_logs() {
    let now = Instant::now();
    let mut c = Convoy::default();
    let mut f = frame();
    f["vehicles"][1]["name"] = json!("Ali\u{202e}\n Kaya");
    f["vehicles"][1]["nameBinding"] = json!("native-session-actor");
    c.accept(&f, now);
    assert_eq!(c.snapshot(now)["players"][0]["name"], "Ali Kaya");
    let native = c.native_snapshot(now);
    assert_eq!(native["players"][0]["id"], "bbbbbbbbbbbb");
    assert_eq!(native["players"][0]["x"], 130.0);
    assert_eq!(native["players"][0]["name"], "Ali Kaya");
    assert!(!serde_json::to_string(&c.logs()).unwrap().contains("Ali"));
    f["vehicles"][1]["nameBinding"] = json!("nearest");
    c.accept(&f, now);
    assert!(c.snapshot(now)["players"][0].get("name").is_none());
}
#[test]
fn invalid_sample_clears_native_world_too() {
    let now = Instant::now();
    let mut c = Convoy::default();
    c.accept(&frame(), now);
    c.accept(&json!({"status":"sample-refused"}), now);
    assert!(c.native_snapshot(now)["players"].as_array().unwrap().is_empty());
}

#[test]
fn replacement_own_actor_requires_three_coherent_samples() {
    let now = Instant::now();
    let mut c = Convoy::default();
    c.accept(&frame(), now);
    let mut f = frame();
    f["vehicles"][0]["id"] = json!("cccccccccccc");
    for _ in 0..2 {
        c.accept(&f, now);
        assert!(c.snapshot(now)["players"].as_array().unwrap().is_empty());
    }
    c.accept(&f, now);
    assert_eq!(c.snapshot(now)["players"].as_array().unwrap().len(), 1);
}

#[test]
fn invalid_gaps_do_not_count_toward_replacement_ownership() {
    let now = Instant::now();
    let mut c = Convoy::default();
    c.accept(&frame(), now);
    let mut f = frame();
    f["vehicles"][0]["id"] = json!("cccccccccccc");
    for _ in 0..3 {
        c.accept(&f, now);
        assert!(c.snapshot(now)["players"].as_array().unwrap().is_empty());
        c.accept(&json!({"status":"sample-refused"}), now);
    }
}
