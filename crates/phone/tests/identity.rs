use std::time::{Duration, Instant};
use westplay_phone::{DeviceInfo, IdentityStore, Invite, Platform};
#[test]
fn stored_identity_survives_port_change_and_forget_revokes_only_credentials() {
    let folder = std::env::temp_dir().join(format!("westplay-phone-test-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let file = folder.join("identity.json");
    let mut first = IdentityStore::load(file.clone()).unwrap();
    let id = first.pc_id().to_owned();
    let pin = first.fingerprint();
    first.remember_port(49123).unwrap();
    let device = DeviceInfo { id: "phone-one".into(), name: "Test".into(), platform: Platform::Android };
    let token = "ab".repeat(32);
    first.pair(device.clone(), &token).unwrap();
    assert!(first.authenticate(&device, &token));
    assert!(!std::fs::read_to_string(&file).unwrap().contains(&token));
    assert!(!std::fs::read_to_string(&file).unwrap().contains("PRIVATE KEY"));
    let mut second = IdentityStore::load(file.clone()).unwrap();
    assert_eq!(id, second.pc_id());
    assert_eq!(pin, second.fingerprint());
    assert_eq!(second.preferred_port(), 49123);
    second.forget().unwrap();
    assert!(!second.authenticate(&device, &token));
    assert_eq!(pin, second.fingerprint());
    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(folder).unwrap();
}
#[test]
fn invites_expire_and_lock_after_five_failed_attempts() {
    let now = Instant::now();
    let mut invite = Invite::new(now);
    let secret = invite.secret().to_owned();
    assert!(!invite.redeem(&secret, now + Duration::from_secs(121)));
    let mut invite = Invite::new(now);
    let secret = invite.secret().to_owned();
    for _ in 0..5 {
        assert!(!invite.redeem("wrong", now));
    }
    assert!(!invite.redeem(&secret, now));
    let mut invite = Invite::new(now);
    let secret = invite.secret().to_owned();
    assert!(invite.redeem(&secret, now));
    assert!(!invite.redeem(&secret, now));
}
