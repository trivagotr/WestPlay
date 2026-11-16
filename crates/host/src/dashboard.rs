//! Fresh matching-process SDK dashboard samples; the main bridge ABI stays unchanged.
use std::time::{Duration, Instant};
use westplay_shared::dashboard::DashboardReader;
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
    reader: Option<(u32, DashboardReader)>,
    tried: Option<Instant>,
    progress: Progress,
}
impl Watch {
    pub fn read(&mut self, pid: u32, now: Instant) -> Option<westplay_shared::dashboard::DashboardTelemetry> {
        if pid == 0 {
            self.reader = None;
            self.progress = Progress::default();
            self.tried = None;
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
            self.reader = DashboardReader::open(pid).ok().map(|reader| (pid, reader));
        }
        let sample = self.reader.as_ref()?.1.read()?;
        if sample.pid != pid || !self.progress.accept(pid, sample.counter, now) {
            return None;
        }
        Some(sample)
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

/// SDK exposes component wear, not an aggregate vehicle damage channel. Require all five components.
pub fn total_damage(wear: [Option<f32>; 5]) -> Option<f64> {
    wear.into_iter()
        .try_fold(0.0, |sum, value| value.filter(|v| v.is_finite() && (0.0..=1.0).contains(v)).map(|v| sum + v as f64))
        .map(|sum| sum / 5.0)
}
/// Flat web contract. Unsupported Coaches and actual lane assist state remain unavailable.
pub fn fields(
    sample: Option<westplay_shared::dashboard::DashboardTelemetry>,
    wear: [Option<f32>; 5],
    coach: bool,
) -> serde_json::Value {
    let number = |index| sample.as_ref().and_then(|s| s.number(index));
    let consumption = number(0).filter(|v| *v >= 0.0).map(|v| v * 100.0);
    let cargo = number(1).filter(|v| (0.0..=1.0).contains(v));
    let speed = number(3).map(|v| v.abs() * 3.6);
    let cruise = number(4).filter(|v| *v > 0.0).map(|v| v * 3.6);
    let active = sample.as_ref().and_then(|s| s.boolean(if coach { 2 } else { 1 }));
    let deadline = (!coach && active == Some(true)).then(|| number(5)).flatten();
    let destination = (!coach && active == Some(true)).then(|| sample.as_ref().and_then(|s| s.destination())).flatten();
    let game_time = number(2).filter(|v| *v >= 0.0);
    let delay = deadline.zip(game_time).map(|(scheduled, now)| (now - scheduled).max(0.0));
    serde_json::json!({
        "speedKph":speed,"cruiseSpeedKph":cruise,"fuelConsumption":consumption,
        "cargoDamage":cargo,"totalDamage":total_damage(wear),"totalDamageSource":"component-average",
        "laneAssist":null,
        "job":{"kind":if coach {"coach"}else{"cargo"},"active":active,"destination":destination,"scheduledArrivalMinutes":deadline,
            "gameTimeMinutes":game_time,"delayMinutes":delay,"passengerCount":null}
    })
}
#[cfg(test)]
mod field_tests {
    use super::*;
    use westplay_shared::dashboard::DashboardTelemetry;
    #[test]
    fn sdk_units_and_unknown_optional_features_remain_distinct() {
        let mut sample = DashboardTelemetry::default();
        sample.set_number(0, Some(0.321));
        sample.set_number(1, Some(0.0));
        sample.set_number(2, Some(1441.0));
        sample.set_number(3, Some(-20.0));
        sample.set_number(4, Some(25.0));
        let result = fields(Some(sample), [Some(0.2); 5], false);
        assert!((result["fuelConsumption"].as_f64().unwrap() - 32.1).abs() < 0.001);
        assert_eq!(result["speedKph"], 72.0);
        assert_eq!(result["cruiseSpeedKph"], 90.0);
        assert_eq!(result["cargoDamage"], 0.0);
        assert_eq!(result["job"]["gameTimeMinutes"], 1441.0);
        assert!(result["laneAssist"].is_null());
        assert!(result["job"]["active"].is_null());
        let result = fields(None, [None; 5], false);
        assert!(result["fuelConsumption"].is_null());
        assert!(result["cargoDamage"].is_null());
    }
    #[test]
    fn job_deadline_is_exact_sdk_time_and_job_end_clears_details() {
        let mut sample = DashboardTelemetry::default();
        sample.set_bool(1, Some(true));
        sample.set_destination(Some("Ankara"));
        sample.set_number(2, Some(1442.0));
        sample.set_number(5, Some(1440.0));
        let value = fields(Some(sample), [None; 5], false);
        assert_eq!(value["job"]["destination"], "Ankara");
        assert_eq!(value["job"]["delayMinutes"], 2.0);
        assert!(fields(Some(sample), [None; 5], true)["job"]["scheduledArrivalMinutes"].is_null());
        sample.set_bool(1, Some(false));
        let value = fields(Some(sample), [None; 5], false);
        assert_eq!(value["job"]["active"], false);
        assert!(value["job"]["destination"].is_null());
        assert!(value["job"]["delayMinutes"].is_null());
    }
    #[test]
    fn aggregate_requires_every_valid_component_and_cruise_must_be_engaged() {
        assert_eq!(total_damage([Some(0.0); 5]), Some(0.0));
        assert_eq!(total_damage([Some(0.2), None, Some(0.2), Some(0.2), Some(0.2)]), None);
        assert_eq!(total_damage([Some(f32::NAN); 5]), None);
        let mut sample = DashboardTelemetry::default();
        sample.set_number(4, Some(0.0));
        assert!(fields(Some(sample), [None; 5], false)["cruiseSpeedKph"].is_null());
    }
}
