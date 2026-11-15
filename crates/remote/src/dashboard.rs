//! Latest-only, expiring dashboard transport. No credentials or process identity in browser data.
use serde_json::{Value, json};
use std::time::{Duration, Instant};
#[derive(Default)]
pub struct Dashboard {
    sample: Option<(u32, Value, Instant)>,
}
impl Dashboard {
    pub fn publish(&mut self, pid: u32, vehicle: Value, now: Instant) {
        self.sample = (pid != 0 && !vehicle.is_null()).then_some((pid, vehicle, now));
    }
    pub fn pid(&self, now: Instant) -> Option<u32> {
        self.sample
            .as_ref()
            .filter(|(_, _, at)| now.saturating_duration_since(*at) <= Duration::from_secs(2))
            .map(|s| s.0)
    }
    pub fn snapshot(&self, now: Instant) -> Value {
        match self.sample.as_ref().filter(|_| self.pid(now).is_some()) {
            Some((_, v, at)) => {
                json!({"connected":true,"ageMs":now.saturating_duration_since(*at).as_millis(),"vehicle":v})
            }
            None => json!({"connected":false,"ageMs":null,"vehicle":null}),
        }
    }
}
