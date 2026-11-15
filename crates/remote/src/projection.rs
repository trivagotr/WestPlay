//! Only screen coordinates cross to the browser. Late camera/source results are discarded.
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapMarker {
    pub marker: u32,
    pub x: f64,
    pub y: f64,
    pub distance: f64,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapProjection {
    pub id: u64,
    pub clip: Option<[f64; 4]>,
    pub markers: Vec<MapMarker>,
}
impl MapProjection {
    fn valid(&self) -> bool {
        if self.markers.len() > 128 {
            return false;
        }
        let Some([x, y, w, h]) = self.clip else { return self.markers.is_empty() };
        if ![x, y, w, h].iter().all(|n| n.is_finite() && (0.0..=1.0).contains(n))
            || w <= 0.0
            || h <= 0.0
            || x + w > 1.000001
            || y + h > 1.000001
        {
            return false;
        }
        let mut ids = std::collections::HashSet::new();
        self.markers.iter().all(|m| {
            m.marker > 0
                && m.marker <= 128
                && ids.insert(m.marker)
                && m.x.is_finite()
                && m.y.is_finite()
                && m.x >= x
                && m.x <= x + w
                && m.y >= y
                && m.y <= y + h
                && m.distance.is_finite()
                && (0.0..=48_000_000.0).contains(&m.distance)
        })
    }
}
#[derive(Default)]
pub struct ProjectionStore {
    next: u64,
    pending: Option<(u64, (u64, u64), Instant)>,
    current: Option<((u64, u64), MapProjection, Instant)>,
}
impl ProjectionStore {
    pub fn clear(&mut self) {
        self.pending = None;
        self.current = None;
    }
    pub fn request(&mut self, key: (u64, u64), now: Instant) -> u64 {
        if self.current.as_ref().is_some_and(|(old, _, _)| *old != key) {
            self.current = None;
        }
        self.next = self.next.wrapping_add(1);
        self.pending = Some((self.next, key, now));
        self.next
    }
    pub fn accept(&mut self, key: (u64, u64), reply: MapProjection, now: Instant) -> bool {
        if !self.pending.is_some_and(|(id, source, at)| {
            id == reply.id && source == key && now.saturating_duration_since(at) <= Duration::from_millis(350)
        }) {
            return false;
        }
        self.pending = None;
        if !reply.valid() {
            self.current = None;
            return false;
        }
        self.current = Some((key, reply, now));
        true
    }
    pub fn get(&self, key: Option<(u64, u64)>, now: Instant) -> Option<&MapProjection> {
        self.current
            .as_ref()
            .filter(|(source, _, at)| {
                Some(*source) == key && now.saturating_duration_since(*at) <= Duration::from_millis(350)
            })
            .map(|(_, reply, _)| reply)
    }
}
