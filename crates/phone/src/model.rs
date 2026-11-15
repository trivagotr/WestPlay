use crate::protocol::*;
use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};

pub struct PhoneUpdate {
    pub revision: u64,
    pub state: PhoneState,
    pub events: Vec<PhoneNotificationEvent>,
}

#[derive(Default)]
pub struct PhoneModel {
    state: PhoneState,
    revision: u64,
    sequence: u64,
    events: VecDeque<PhoneNotificationEvent>,
    seen: HashMap<String, (Notification, Instant)>,
}
impl PhoneModel {
    pub fn state(&self) -> &PhoneState {
        &self.state
    }
    pub fn connect(&mut self, device: DeviceInfo) -> u64 {
        self.state.generation += 1;
        self.state.connected = true;
        self.state.device_id = Some(device.id);
        self.state.device_name = Some(device.name);
        self.state.platform = Some(device.platform);
        self.state.battery_pct = None;
        self.state.charging = None;
        self.state.capabilities = Capabilities::default();
        self.sequence = 0;
        self.events.clear();
        self.seen.clear();
        self.revision += 1;
        self.state.generation
    }
    pub fn disconnect(&mut self, generation: u64) {
        if self.state.generation != generation || !self.state.connected {
            return;
        }
        self.state.connected = false;
        self.state.battery_pct = None;
        self.state.charging = None;
        self.state.capabilities = Capabilities::default();
        self.events.clear();
        self.seen.clear();
        self.revision += 1;
    }
    pub fn forget(&mut self) {
        let generation = self.state.generation + 1;
        self.state = PhoneState { generation, ..Default::default() };
        self.events.clear();
        self.seen.clear();
        self.revision += 1;
    }
    pub fn apply(&mut self, generation: u64, envelope: Envelope) -> bool {
        if !self.state.connected || self.state.generation != generation || envelope.sequence <= self.sequence {
            return false;
        }
        self.sequence = envelope.sequence;
        match envelope.event {
            Event::Hello(caps) => {
                if !caps.notifications {
                    self.events.clear();
                    self.seen.clear();
                }
                self.state.capabilities = caps;
                self.revision += 1;
            }
            Event::State { battery_pct, charging } => {
                if self.state.battery_pct != battery_pct || self.state.charging != charging {
                    self.state.battery_pct = battery_pct;
                    self.state.charging = charging;
                    self.revision += 1;
                }
            }
            Event::Notification { updated, mut data } if self.state.capabilities.notifications => {
                if !self.state.capabilities.notification_body {
                    data.body = None;
                }
                let now = Instant::now();
                if self
                    .seen
                    .get(&data.id)
                    .is_some_and(|(old, at)| old == &data && at.elapsed() < Duration::from_secs(1))
                {
                    return true;
                }
                if self.seen.len() >= 100 {
                    if let Some(oldest) = self.seen.iter().min_by_key(|(_, (_, at))| *at).map(|(id, _)| id.clone()) {
                        self.seen.remove(&oldest);
                    }
                }
                self.seen.insert(data.id.clone(), (data.clone(), now));
                self.events.retain(|e| e.id != data.id);
                while self.events.len() >= 3 {
                    self.events.pop_front();
                }
                self.events.push_back(PhoneNotificationEvent {
                    kind: if updated { "updated" } else { "posted" }.into(),
                    id: data.id,
                    app_id: Some(data.app_id),
                    app_name: Some(data.app_name),
                    title: Some(data.title),
                    body: data.body,
                });
                self.revision += 1;
            }
            Event::Removed(id) => {
                self.seen.remove(&id);
                self.events.retain(|e| e.id != id);
                while self.events.len() >= 3 {
                    self.events.pop_front();
                }
                self.events.push_back(PhoneNotificationEvent {
                    kind: "removed".into(),
                    id,
                    app_id: None,
                    app_name: None,
                    title: None,
                    body: None,
                });
                self.revision += 1;
            }
            Event::End => self.disconnect(generation),
            _ => {}
        }
        true
    }
    pub fn take_after(&mut self, revision: u64) -> Option<PhoneUpdate> {
        if self.revision == revision {
            return None;
        }
        Some(PhoneUpdate {
            revision: self.revision,
            state: self.state.clone(),
            events: self.events.drain(..).collect(),
        })
    }
}
