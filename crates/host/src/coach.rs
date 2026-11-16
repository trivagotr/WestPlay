//! Fresh matching-process Coaches samples. Main game bridge compatibility is unchanged.
use std::time::{Duration, Instant};
use westplay_shared::coach::CoachReader;
#[derive(Default)]
struct Progress {
    last: Option<(u32, u64, Instant, bool)>,
}
impl Progress {
    fn accept(&mut self, pid: u32, counter: u64, now: Instant) -> bool {
        match self.last {
            Some((old, seen, _, _)) if old != pid || counter < seen => self.last = Some((pid, counter, now, false)),
            Some((_, seen, _, _)) if counter > seen => self.last = Some((pid, counter, now, true)),
            None => self.last = Some((pid, counter, now, false)),
            _ => {}
        }
        self.last
            .is_some_and(|(_, _, at, changed)| changed && now.saturating_duration_since(at) <= Duration::from_secs(2))
    }
}
#[derive(Default)]
pub struct Watch {
    reader: Option<(u32, CoachReader)>,
    tried: Option<Instant>,
    progress: Progress,
}
impl Watch {
    pub fn read(&mut self, pid: u32, now: Instant) -> Option<serde_json::Value> {
        if pid == 0 {
            return None;
        }
        if self.reader.as_ref().is_some_and(|(old, _)| *old != pid) {
            self.reader = None;
            self.progress = Progress::default();
            self.tried = None;
        }
        if self.reader.is_none()
            && self.tried.is_none_or(|at| now.saturating_duration_since(at) >= Duration::from_secs(1))
        {
            self.tried = Some(now);
            self.reader = CoachReader::open(pid).ok().map(|reader| (pid, reader));
        }
        let sample = self.reader.as_ref()?.1.read()?;
        if sample.pid != pid || !self.progress.accept(pid, sample.counter, now) {
            return None;
        }
        Some(
            serde_json::json!({"supported":sample.supported&1!=0,"satisfaction":sample.satisfaction().map(|v|v*100.0)}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stalled_or_new_process_cannot_reuse_old_passenger_values() {
        let now = Instant::now();
        let mut p = Progress::default();
        assert!(!p.accept(7, 1, now));
        assert!(p.accept(7, 2, now));
        assert!(!p.accept(7, 2, now + Duration::from_millis(2001)));
        assert!(p.accept(7, 3, now + Duration::from_millis(2100)));
        assert!(!p.accept(8, 1, now + Duration::from_millis(2200)));
        assert!(!p.accept(8, 0, now + Duration::from_millis(2300)));
        assert!(p.accept(8, 1, now + Duration::from_millis(2400)));
    }
}
