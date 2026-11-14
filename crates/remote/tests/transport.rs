use std::time::Instant;
use westplay_remote::{AccessUnits, Input, InputQueue, RemoteControl};

#[test]
fn annex_b_fragmentation_preserves_parameter_sets_and_access_units() {
    let a = [0, 0, 0, 1, 9, 0xf0, 0, 0, 1, 0x67, 1, 2, 0, 0, 1, 0x68, 3, 0, 0, 1, 0x65, 4];
    let b = [0, 0, 0, 1, 9, 0xf0, 0, 0, 1, 0x41, 5];
    let mut parser = AccessUnits::default();
    let mut got = Vec::new();
    for v in a.into_iter().chain(b) {
        got.extend(parser.push(&[v]));
    }
    got.extend(parser.finish());
    assert_eq!(got.len(), 2);
    assert_eq!(got[0], a);
    assert_eq!(got[1], b);
}

#[test]
fn queue_overflow_releases_and_drops_rest_of_broken_gesture() {
    let mut q = InputQueue::default();
    for _ in 0..140 {
        q.push(Input::Move { x: 0.5, y: 0.5 });
    }
    assert!(matches!(q.take().first(), Some(Input::Cancel)));
    q.push(Input::Move { x: 0.4, y: 0.4 });
    assert!(q.take().is_empty());
    q.push(Input::Cancel);
    q.push(Input::Down { x: 0.2, y: 0.3 });
    assert!(matches!(q.take().last(), Some(Input::Down { .. })));
}

#[test]
fn lifting_after_overflow_allows_the_next_gesture() {
    let mut q = InputQueue::default();
    for _ in 0..140 {
        q.push(Input::Move { x: 0.5, y: 0.5 });
    }
    q.take();
    q.push(Input::Up { x: 0.5, y: 0.5 });
    q.push(Input::Down { x: 0.3, y: 0.4 });
    assert!(matches!(q.take().last(), Some(Input::Down { .. })));
}

#[test]
fn off_by_default_and_rejects_public_or_unassigned_interfaces() {
    let remote = RemoteControl::new();
    assert!(!remote.status().enabled);
    assert!(!remote.wants_frames());
    assert!(remote.configure(Some("8.8.8.8".parse().unwrap())).is_err());
    assert!(!remote.status().enabled);
    remote.configure(None).unwrap();
    assert!(remote.take_input(Instant::now()).is_empty());
}
