use westplay_phone::{DeviceInfo, PhoneModel, Platform, parse_event};
#[test]
fn validates_cross_platform_events_and_limits() {
    assert!(
        parse_event(
            r#"{"version":1,"sequence":1,"eventType":"deviceState","payload":{"batteryPct":0,"charging":false}}"#
        )
        .is_ok()
    );
    assert!(
        parse_event(
            r#"{"version":1,"sequence":1,"eventType":"deviceState","payload":{"batteryPct":null,"charging":null}}"#
        )
        .is_ok()
    );
    for text in [
        r#"{"version":2,"sequence":1,"eventType":"heartbeat","payload":{}}"#,
        r#"{"version":1,"sequence":0,"eventType":"heartbeat","payload":{}}"#,
        r#"{"version":1,"sequence":1,"eventType":"deviceState","payload":{"batteryPct":101}}"#,
    ] {
        assert!(parse_event(text).is_err());
    }
    let fixture = include_str!("../../../protocol/fixtures/ios-capabilities.json");
    assert!(parse_event(fixture).is_ok());
    let too_big=serde_json::json!({"version":1,"sequence":2,"eventType":"notificationPosted","payload":{"id":"a","appId":"com.test","appName":"Test","title":"x".repeat(161)}}).to_string();
    assert!(parse_event(&too_big).is_err());
}
#[test]
fn session_epoch_is_distinct_from_state_revision_and_stale_session_cannot_clear_new() {
    let mut model = PhoneModel::default();
    let d = DeviceInfo { id: "device".into(), name: "Phone".into(), platform: Platform::Android };
    let epoch = model.connect(d.clone());
    model.apply(
        epoch,
        parse_event(
            r#"{"version":1,"sequence":1,"eventType":"deviceState","payload":{"batteryPct":12,"charging":false}}"#,
        )
        .unwrap(),
    );
    assert_eq!(model.state().generation, epoch);
    assert_eq!(model.state().battery_pct, Some(12));
    let newer = model.connect(d);
    model.disconnect(epoch);
    assert!(model.state().connected);
    model.disconnect(newer);
    assert!(!model.state().connected);
    assert_eq!(model.state().battery_pct, None);
}
#[test]
fn duplicate_sequence_and_permission_loss_do_not_replay_notifications() {
    let mut model = PhoneModel::default();
    let epoch = model.connect(DeviceInfo { id: "d".into(), name: "P".into(), platform: Platform::Android });
    model.apply(
        epoch,
        parse_event(
            r#"{"version":1,"sequence":1,"eventType":"deviceHello","payload":{"capabilities":{"notifications":true}}}"#,
        )
        .unwrap(),
    );
    let event = r#"{"version":1,"sequence":2,"eventType":"notificationPosted","payload":{"id":"a","appId":"x","appName":"X","title":"Hello","body":"private"}}"#;
    model.apply(epoch, parse_event(event).unwrap());
    model.apply(epoch, parse_event(event).unwrap());
    let update = model.take_after(0).unwrap();
    assert_eq!(update.events.len(), 1);
    assert!(update.events[0].body.is_none());
    model.apply(epoch,parse_event(r#"{"version":1,"sequence":3,"eventType":"deviceHello","payload":{"capabilities":{"notifications":false}}}"#).unwrap());
    assert!(model.take_after(update.revision).unwrap().events.is_empty());
}
