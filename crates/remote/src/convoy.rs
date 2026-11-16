//! Nearby Convoy samples; static native bindings, not an authenticated or complete roster.
use serde_json::{Value, json};
use std::{
    collections::{HashSet, VecDeque},
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct Convoy {
    players: Vec<Value>,
    world: Vec<Value>,
    at: Option<Instant>,
    own_id: Option<String>,
    pending_own: Option<(String, u8)>,
    pub status: String,
    events: VecDeque<Value>,
    samples: u64,
    gaps: u64,
}
fn number(v: &Value, key: &str) -> Option<f64> {
    v.get(key)?.as_f64().filter(|n| n.is_finite() && n.abs() < 17_000_000.0)
}
fn reason(v: &Value) -> &str {
    v.as_str()
        .filter(|s| {
            s.len() <= 64
                && !s.is_empty()
                && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
        .unwrap_or("invalid-sample")
}
fn native_name(row: &Value) -> Option<String> {
    if row["nameBinding"] != "native-session-actor" {
        return None;
    }
    let name = row["name"].as_str()?;
    if name.len() > 256 {
        return None;
    }
    let clean: String = name
        .chars()
        .filter(|c| {
            !c.is_control()
                && !matches!(*c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}' | '\u{200e}' | '\u{200f}')
        })
        .collect();
    let clean = clean.trim();
    (!clean.is_empty() && clean.chars().count() <= 80 && !clean.starts_with("@@")).then(|| clean.to_owned())
}
fn named(mut row: Value, name: &Option<String>) -> Value {
    if let Some(name) = name {
        row["name"] = json!(name);
    }
    row
}
impl Convoy {
    pub fn event(&mut self, kind: &str, details: Value) {
        if self.events.len() >= 256 {
            self.events.pop_front();
        }
        self.events.push_back(json!({"timeMs":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis(),"event":kind,"details":details}));
    }
    pub fn clear(&mut self, status: &str) {
        self.players.clear();
        self.world.clear();
        self.at = None;
        self.status = status.into();
        self.event(status, json!({}));
    }
    pub fn accept(&mut self, v: &Value, now: Instant) {
        self.players.clear();
        self.world.clear();
        self.at = Some(now);
        self.status.clear();
        self.samples += 1;
        let result = (|| -> Option<Vec<Value>> {
            if v["status"] != "candidate-sample" {
                return None;
            }
            let rows = v["vehicles"].as_array()?;
            if rows.len() > 128 {
                return None;
            }
            let own = &v["ownTelemetry"];
            let ox = number(own, "x")?;
            let oz = number(own, "z")?;
            let mut ids = HashSet::new();
            let mut positions = Vec::new();
            for row in rows {
                let id = row["id"].as_str()?;
                if id.len() != 12 || !id.bytes().all(|b| b.is_ascii_hexdigit()) || !ids.insert(id) {
                    return None;
                }
                positions.push((id, number(row, "x")?, number(row, "z")?, native_name(row)));
            }
            let matching: Vec<_> = positions.iter().filter(|(_, x, z, _)| (x - ox).hypot(z - oz) <= 0.5).collect();
            if matching.len() != 1 {
                return None;
            }
            let own_id = matching[0].0;
            if self.own_id.as_ref().is_some_and(|id| id != own_id) {
                let count = self.pending_own.as_ref().filter(|(id, _)| id == own_id).map_or(1, |(_, n)| n + 1);
                self.pending_own = Some((own_id.into(), count));
                if count < 3 {
                    self.status = "ownership-changed".into();
                    return None;
                }
            }
            self.own_id = Some(own_id.into());
            self.pending_own = None;
            self.status.clear();
            self.world = positions
                .iter()
                .filter(|(id, _, _, _)| *id != own_id)
                .enumerate()
                .map(|(i, (id, x, z, name))| {
                    named(json!({"id":id,"marker":i+1,"x":x,"z":z,"distance":(x-ox).hypot(z-oz)}), name)
                })
                .collect();
            Some(
                positions
                    .iter()
                    .filter(|(id, _, _, _)| *id != own_id)
                    .enumerate()
                    .map(|(i, (_, x, z, name))| {
                        named(json!({"marker":i+1,"dx":x-ox,"dz":z-oz,"distance":(x-ox).hypot(z-oz)}), name)
                    })
                    .collect(),
            )
        })();
        let valid = result.is_some();
        if !valid && self.status != "ownership-changed" {
            self.pending_own = None;
        }
        self.players = result.unwrap_or_default();
        if !valid || self.players.is_empty() {
            self.gaps += 1;
        }
        if self.status != "ownership-changed" {
            self.status = if valid { "observing" } else { "sample-unavailable" }.into();
        }
        self.event("sample",json!({"sample":self.samples,"accepted":valid,"visible":self.players.len(),"gaps":self.gaps,"namedPlayers":self.players.iter().filter(|r|r.get("name").is_some()).count(),"nameStatus":reason(&v["nameStatus"]),"readMs":v["readMs"].as_f64().filter(|v|v.is_finite()&&*v>=0.0&&*v<10000.0),"reason":if valid{"ok"}else{reason(&v["reason"])}}));
    }
    pub fn snapshot(&self, now: Instant) -> Value {
        let fresh = self.at.is_some_and(|at| now.saturating_duration_since(at) <= Duration::from_secs(2));
        json!({"status":self.status,"fresh":fresh,"players":if fresh{self.players.clone()}else{vec![]},"samples":self.samples,"gaps":self.gaps,"completeRosterVerified":false})
    }
    pub fn native_snapshot(&self, now: Instant) -> Value {
        let fresh = self.at.is_some_and(|at| now.saturating_duration_since(at) <= Duration::from_secs(2));
        json!({"type":"convoyPlayers","status":self.status,"fresh":fresh,"players":if fresh {self.world.clone()}else{vec![]},"identityVerified":false})
    }
    pub fn logs(&self) -> Vec<Value> {
        self.events.iter().cloned().collect()
    }
}
pub(crate) struct Observer {
    pub data: std::sync::Mutex<Convoy>,
    generation: AtomicU64,
}
impl Default for Observer {
    fn default() -> Self {
        Self {
            data: std::sync::Mutex::new(Convoy { status: "waiting-game".into(), ..Convoy::default() }),
            generation: AtomicU64::new(0),
        }
    }
}
impl Observer {
    pub(crate) fn is_current(&self, generation: u64) -> bool {
        self.generation.load(Ordering::Acquire) == generation
    }
    #[cfg(test)]
    pub(crate) fn begin(&self) -> u64 {
        self.begin_when(|| true).unwrap()
    }
    pub(crate) fn begin_when(&self, allowed: impl FnOnce() -> bool) -> Option<u64> {
        let mut data = self.data.lock().unwrap();
        if !allowed() {
            return None;
        }
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let events = std::mem::take(&mut data.events);
        let (samples, gaps) = (data.samples, data.gaps);
        *data = Convoy { events, samples, gaps, ..Convoy::default() };
        data.clear("starting");
        Some(generation)
    }
    pub(crate) fn with_current<R>(&self, generation: u64, action: impl FnOnce(&mut Convoy) -> R) -> Option<R> {
        let mut data = self.data.lock().unwrap();
        if self.generation.load(Ordering::Acquire) != generation {
            return None;
        }
        Some(action(&mut data))
    }
    pub fn stop(&self) {
        let mut data = self.data.lock().unwrap();
        self.generation.fetch_add(1, Ordering::AcqRel);
        data.clear("stopped");
    }
}
#[cfg(test)]
mod ownership_tests {
    use super::*;
    use std::sync::Arc;
    #[test]
    fn cancelled_start_cannot_run_a_spawn_or_mutation() {
        let o = Observer::default();
        let generation = o.begin();
        o.stop();
        let mut ran = false;
        assert!(o.with_current(generation, |_| { ran = true }).is_none());
        assert!(!ran);
    }
    #[test]
    fn cancelled_target_cannot_replace_waiting_status() {
        let o = Observer::default();
        o.data.lock().unwrap().clear("waiting-game");
        assert!(o.begin_when(|| false).is_none());
        assert_eq!(o.data.lock().unwrap().status, "waiting-game");
    }
    #[test]
    fn concurrent_starts_have_unique_generations() {
        let o = Arc::new(Observer::default());
        let threads: Vec<_> = (0..16)
            .map(|_| {
                let o = o.clone();
                std::thread::spawn(move || o.begin())
            })
            .collect();
        let mut ids: Vec<_> = threads.into_iter().map(|t| t.join().unwrap()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 16);
        let current = o.generation.load(Ordering::Acquire);
        assert!(o.with_current(current, |d| d.status.clone()).is_some());
    }
}
