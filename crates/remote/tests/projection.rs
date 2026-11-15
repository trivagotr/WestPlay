use std::time::{Duration, Instant};
use westplay_remote::projection::{MapMarker, MapProjection, ProjectionStore};
fn reply(id: u64) -> MapProjection {
    MapProjection {
        id,
        clip: Some([0.2, 0.1, 0.7, 0.8]),
        markers: vec![MapMarker { marker: 1, x: 0.5, y: 0.4, distance: 50.0 }],
    }
}
#[test]
fn rejects_old_source_and_expires_screen_points() {
    let now = Instant::now();
    let mut s = ProjectionStore::default();
    let id = s.request((1, 1), now);
    assert!(s.accept((1, 1), reply(id), now));
    assert!(!s.accept((1, 1), reply(id), now + Duration::from_millis(100)));
    assert!(s.get(Some((1, 1)), now).is_some());
    let next = s.request((1, 2), now);
    assert!(s.get(Some((1, 2)), now).is_none());
    assert!(!s.accept((1, 2), reply(id), now));
    assert!(s.accept((1, 2), reply(next), now));
    assert!(s.get(Some((1, 2)), now + Duration::from_millis(351)).is_none());
    assert!(s.get(None, now).is_none());
}
#[test]
fn invalid_or_hidden_map_clears_markers() {
    let now = Instant::now();
    let mut s = ProjectionStore::default();
    let id = s.request((1, 1), now);
    let mut v = reply(id);
    v.markers[0].x = 1.5;
    assert!(!s.accept((1, 1), v, now));
    assert!(s.get(Some((1, 1)), now).is_none());
    let id = s.request((1, 1), now);
    let hidden = MapProjection { id, clip: None, markers: vec![] };
    assert!(s.accept((1, 1), hidden, now));
    assert!(s.get(Some((1, 1)), now).unwrap().markers.is_empty());
}
