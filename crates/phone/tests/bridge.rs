use std::time::{Duration, Instant};
use westplay_phone::{PhoneBridge, interfaces};
#[test]
fn data_bridge_is_off_by_default_and_remembers_identity_and_available_port() {
    let folder = std::env::temp_dir().join(format!("westplay-phone-bridge-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("identity.json");
    let bridge = PhoneBridge::new(path.clone()).unwrap();
    assert!(!bridge.status().enabled);
    assert!(bridge.configure(Some("8.8.8.8".parse().unwrap())).is_err());
    let Some(ip) = interfaces().first().map(|i| i.address) else { return };
    bridge.configure(Some(ip)).unwrap();
    let wait = Instant::now();
    while bridge.status().port.is_none() && wait.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    let first = bridge.status();
    assert!(first.enabled);
    assert!(first.port.is_some());
    assert!(first.pair_uri.as_deref().unwrap().starts_with("westplay://phone?"));
    bridge.configure(None).unwrap();
    assert!(!bridge.status().enabled);
    bridge.configure(Some(ip)).unwrap();
    let wait = Instant::now();
    while bridge.status().port.is_none() && wait.elapsed() < Duration::from_secs(5) {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(first.pc_id, bridge.status().pc_id);
    assert_eq!(first.port, bridge.status().port);
    bridge.configure(None).unwrap();
    drop(bridge);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
