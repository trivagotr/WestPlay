use std::time::{Duration, Instant};
use westplay_remote::{Input, Latest, Pairing, allowed_address, valid_origin};
#[test]
fn pairing_expires_and_tokens_cannot_be_replayed_after_reset() {
    let now = Instant::now();
    let mut p = Pairing::new(now);
    let code = p.code().to_string();
    assert!(p.pair(&code, now + Duration::from_secs(121)).is_err());
    let mut p = Pairing::new(now);
    let token = p.pair(&p.code().to_string(), now).unwrap();
    assert!(p.authorized(&token, now));
    assert!(p.pair(&p.code().to_string(), now).is_err());
    p.reset(now);
    assert!(!p.authorized(&token, now));
}
#[test]
fn fifth_bad_code_locks_even_the_correct_code() {
    let now = Instant::now();
    let mut p = Pairing::new(now);
    let code = p.code().to_string();
    for _ in 0..5 {
        assert!(p.pair("wrong", now).is_err());
    }
    assert!(p.pair(&code, now).is_err());
}
#[test]
fn coordinates_and_text_are_bounded() {
    assert!(Input::Down { x: 0.0, y: 1.0 }.valid());
    assert!(!Input::Move { x: f64::NAN, y: 0.0 }.valid());
    assert!(!Input::Down { x: -0.01, y: 0.5 }.valid());
    assert!(!Input::Wheel { x: 0.5, y: 0.5, delta: f64::INFINITY }.valid());
    assert!(!Input::Text { text: "x".repeat(1025) }.valid());
    assert!(Input::Cancel.valid());
    assert!(Input::Key { key: "backspace".into() }.valid());
    assert!(!Input::Key { key: "system".into() }.valid());
}
#[test]
fn slow_reader_only_gets_latest_frame() {
    let mut latest = Latest::default();
    for n in 0..100 {
        latest.put(n);
    }
    assert_eq!(latest.take(), Some(99));
    assert_eq!(latest.take(), None);
}
#[test]
fn only_private_ipv4_and_exact_same_origin_are_accepted() {
    assert!(allowed_address("192.168.1.10".parse().unwrap()));
    for ip in ["0.0.0.0", "127.0.0.1", "8.8.8.8", "169.254.1.3"] {
        assert!(!allowed_address(ip.parse().unwrap()));
    }
    assert!(valid_origin("192.168.1.10:8123", "http://192.168.1.10:8123"));
    assert!(!valid_origin("192.168.1.10:8123", "https://evil.test"));
    assert!(!valid_origin("192.168.1.10:8123", "null"));
}

#[test]
fn qr_invitation_pairs_without_code_and_survives_rescans_but_not_reset() {
    let now = Instant::now();
    let mut p = Pairing::new(now);
    let invite = p.invitation().to_owned();
    assert_eq!(invite.len(), 48);
    assert_ne!(invite, p.code());
    assert_eq!(p.invitation_url("http://192.168.1.5:8123/"), format!("http://192.168.1.5:8123/#pair={invite}"));
    for _ in 0..5 {
        assert!(p.pair("wrong", now).is_err());
    }
    let token = p.pair(&invite, now + Duration::from_secs(121)).unwrap();
    assert!(p.authorized(&token, now + Duration::from_secs(122)));
    assert_eq!(p.pair(&invite, now + Duration::from_secs(123)).unwrap(), token);
    assert!(p.pair(&invite, now + Duration::from_secs(43201)).is_err());
    p.reset(now);
    assert!(p.pair(&invite, now).is_err());
    assert!(!p.authorized(&token, now));
}
