use std::time::{Duration, Instant};
use westplay_remote::dashboard::Dashboard;
#[test]
fn frozen_host_expires_all_vehicle_data_and_internal_pid() {
    let now = Instant::now();
    let mut d = Dashboard::default();
    d.publish(42, serde_json::json!({"parkingBrake":false}), now);
    assert_eq!(d.snapshot(now)["vehicle"]["parkingBrake"], false);
    assert_eq!(d.pid(now), Some(42));
    assert!(d.snapshot(now + Duration::from_millis(2001))["vehicle"].is_null());
    assert_eq!(d.pid(now + Duration::from_millis(2001)), None);
    assert!(!d.snapshot(now).to_string().contains("42"));
}
#[test]
fn closing_game_clears_the_old_snapshot() {
    let now = Instant::now();
    let mut d = Dashboard::default();
    d.publish(7, serde_json::json!({"fuel":20}), now);
    d.publish(0, serde_json::Value::Null, now);
    assert!(d.snapshot(now)["vehicle"].is_null());
    assert_eq!(d.pid(now), None);
}
