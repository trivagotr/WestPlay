//! Opt-in nearby Convoy research. Never a complete or authenticated session roster.
use serde_json::{Value, json};
use std::{
    collections::{HashSet, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::io::{AsyncBufReadExt, BufReader};
#[derive(Default)]
pub struct Convoy {
    players: Vec<Value>,
    world: Vec<Value>,
    at: Option<Instant>,
    own_id: Option<String>,
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
        self.at = Some(now);
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
                positions.push((id, number(row, "x")?, number(row, "z")?));
            }
            let matching: Vec<_> = positions.iter().filter(|(_, x, z)| (x - ox).hypot(z - oz) <= 0.5).collect();
            if matching.len() != 1 {
                return None;
            }
            let own_id = matching[0].0;
            if self.own_id.as_ref().is_some_and(|id| id != own_id) {
                self.status = "ownership-changed".into();
                return None;
            }
            self.own_id = Some(own_id.into());
            self.world = positions
                .iter()
                .filter(|(id, _, _)| *id != own_id)
                .enumerate()
                .map(|(i, (_, x, z))| json!({"marker":i+1,"x":x,"z":z,"distance":(x-ox).hypot(z-oz)}))
                .collect();
            Some(
                positions
                    .iter()
                    .filter(|(id, _, _)| *id != own_id)
                    .enumerate()
                    .map(|(i, (_, x, z))| json!({"marker":i+1,"dx":x-ox,"dz":z-oz,"distance":(x-ox).hypot(z-oz)}))
                    .collect(),
            )
        })();
        let valid = result.is_some();
        self.players = result.unwrap_or_default();
        if !valid || self.players.is_empty() {
            self.gaps += 1;
        }
        if self.status != "ownership-changed" {
            self.status = if valid { "observing" } else { "sample-unavailable" }.into();
        }
        self.event("sample",json!({"sample":self.samples,"accepted":valid,"visible":self.players.len(),"gaps":self.gaps,"readMs":v["readMs"].as_f64().filter(|v|v.is_finite()&&*v>=0.0&&*v<10000.0),"reason":if valid{"ok"}else{reason(&v["reason"])}}));
    }
    pub fn snapshot(&self, now: Instant) -> Value {
        let fresh = self.at.is_some_and(|at| now.saturating_duration_since(at) <= Duration::from_secs(2));
        json!({"status":self.status,"fresh":fresh,"players":if fresh{self.players.clone()}else{vec![]},"samples":self.samples,"gaps":self.gaps,"completeRosterVerified":false})
    }
    pub(crate) fn projection_points(&self, now: Instant) -> Option<(u64, Vec<Value>)> {
        (self.status == "observing"
            && !self.world.is_empty()
            && self.at.is_some_and(|at| now.saturating_duration_since(at) <= Duration::from_secs(2)))
        .then(|| (self.samples, self.world.clone()))
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
        Self { data: std::sync::Mutex::new(Convoy::default()), generation: AtomicU64::new(0) }
    }
}
impl Observer {
    pub(crate) fn projection_source(&self, now: Instant) -> Option<((u64, u64), Vec<Value>)> {
        let data = self.data.lock().unwrap();
        let (sample, points) = data.projection_points(now)?;
        Some(((self.generation.load(Ordering::Acquire), sample), points))
    }
    fn begin(&self) -> u64 {
        let mut data = self.data.lock().unwrap();
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        *data = Convoy::default();
        data.clear("starting");
        generation
    }
    fn with_current<R>(&self, generation: u64, action: impl FnOnce(&mut Convoy) -> R) -> Option<R> {
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
pub(crate) fn start(state: Arc<crate::control::Run>, pid: u32) {
    let generation = state.observer.begin();
    tokio::spawn(async move {
        // Launch under the same ownership lock used by stop/start. A cancelled task cannot spawn.
        let child = state.observer.with_current(generation, |data| {
            if state.stop.load(Ordering::Acquire) || !state.connected.load(Ordering::Acquire) {
                data.clear("stopped");
                return None;
            }
            let path =
                std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.join("WestPlay.ConvoyReader.exe")));
            let Some(path) = path.filter(|p| p.is_file()) else {
                data.clear("reader-missing");
                return None;
            };
            let mut command = tokio::process::Command::new(path);
            command
                .args(["--observe", "--pid", &pid.to_string(), "--seconds", "120"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .kill_on_drop(true);
            #[cfg(windows)]
            command.creation_flags(0x08000000);
            match command.spawn() {
                Ok(child) => Some(child),
                Err(_) => {
                    data.clear("reader-start-failed");
                    None
                }
            }
        });
        let Some(Some(mut child)) = child else { return };
        let Some(stdout) = child.stdout.take() else { return };
        let mut lines = BufReader::new(stdout).lines();
        let begin = Instant::now();
        let mut tick = tokio::time::interval(Duration::from_millis(250));
        loop {
            tokio::select! {
             line=lines.next_line()=>{match line{Ok(Some(line)) if line.len()<=65536=>{
              let keep=state.observer.with_current(generation,|data|{
               match serde_json::from_str::<Value>(&line){
                Ok(v) if v["status"]=="candidate-sample"||v["status"]=="sample-refused"=>{data.accept(&v,Instant::now());true},
                Ok(v) if v["status"]=="refused"=>{data.clear(reason(&v["reason"]));false},
                Ok(v) if v["status"]=="observation-finished"=>{data.clear("finished");false},
                Ok(_)=>true,Err(_)=>{data.clear("reader-invalid-output");false}
               }
              });if keep!=Some(true){break}
             },_=>break}},
             _=tick.tick()=>{if state.stop.load(Ordering::Acquire)||!state.connected.load(Ordering::Acquire)||state.heartbeat.lock().unwrap().elapsed()>Duration::from_secs(5)||state.dashboard.lock().unwrap().pid(Instant::now())!=Some(pid)||state.observer.generation.load(Ordering::Acquire)!=generation||begin.elapsed()>Duration::from_secs(121){break}}
            }
        }
        let _ = child.kill().await;
        let _ = child.wait().await;
        state.observer.with_current(generation, |d| {
            if matches!(d.status.as_str(), "starting" | "observing" | "sample-unavailable") {
                d.clear("finished");
            }
        });
    });
}
#[cfg(test)]
mod ownership_tests {
    use super::*;
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
