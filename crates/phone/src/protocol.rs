use serde::{Deserialize, Serialize};

pub const MAX_MESSAGE: usize = 8192;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    Android,
    Ios,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub platform: Platform,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Capabilities {
    pub battery: bool,
    pub charging: bool,
    pub notifications: bool,
    pub notification_body: bool,
    pub background_relay: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneState {
    pub generation: u64,
    pub connected: bool,
    pub device_id: Option<String>,
    pub device_name: Option<String>,
    pub platform: Option<Platform>,
    pub battery_pct: Option<u8>,
    pub charging: Option<bool>,
    pub capabilities: Capabilities,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: String,
    pub app_id: String,
    pub app_name: String,
    pub title: String,
    pub body: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PhoneNotificationEvent {
    pub kind: String,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Clone, Debug)]
pub enum Event {
    Hello(Capabilities),
    State { battery_pct: Option<u8>, charging: Option<bool> },
    Notification { updated: bool, data: Notification },
    Removed(String),
    Heartbeat,
    End,
}
pub struct Envelope {
    pub sequence: u64,
    pub event: Event,
}

pub fn text_valid(value: &str, max: usize) -> bool {
    value.chars().count() <= max && !value.chars().any(|c| c.is_control() && c != '\n' && c != '\t')
}

pub fn parse_event(text: &str) -> Result<Envelope, String> {
    if text.len() > MAX_MESSAGE {
        return Err("Message too large".into());
    }
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Wire {
        version: u32,
        sequence: u64,
        event_type: String,
        payload: serde_json::Value,
    }
    let wire: Wire = serde_json::from_str(text).map_err(|_| "Invalid event JSON")?;
    if wire.version != 1 || wire.sequence == 0 || wire.sequence > 9_007_199_254_740_991 {
        return Err("Unsupported version or sequence".into());
    }
    let event = match wire.event_type.as_str() {
        "deviceHello" => {
            let caps: Capabilities =
                serde_json::from_value(wire.payload.get("capabilities").cloned().ok_or("Missing capabilities")?)
                    .map_err(|_| "Invalid capabilities")?;
            Event::Hello(caps)
        }
        "deviceState" => {
            #[derive(Deserialize)]
            #[serde(rename_all = "camelCase")]
            struct State {
                battery_pct: Option<u8>,
                charging: Option<bool>,
            }
            let state: State = serde_json::from_value(wire.payload).map_err(|_| "Invalid battery state")?;
            if state.battery_pct.is_some_and(|p| p > 100) {
                return Err("Invalid battery level".into());
            }
            Event::State { battery_pct: state.battery_pct, charging: state.charging }
        }
        "notificationPosted" | "notificationUpdated" => {
            let data: Notification = serde_json::from_value(wire.payload).map_err(|_| "Invalid notification")?;
            if data.id.is_empty()
                || !text_valid(&data.id, 160)
                || !text_valid(&data.app_id, 255)
                || !text_valid(&data.app_name, 80)
                || !text_valid(&data.title, 160)
                || data.body.as_deref().is_some_and(|v| !text_valid(v, 500))
            {
                return Err("Notification fields exceed limits".into());
            }
            Event::Notification { updated: wire.event_type == "notificationUpdated", data }
        }
        "notificationRemoved" => {
            let id = wire.payload.get("id").and_then(|v| v.as_str()).ok_or("Missing notification id")?;
            if id.is_empty() || !text_valid(id, 160) {
                return Err("Invalid notification id".into());
            }
            Event::Removed(id.to_owned())
        }
        "heartbeat" if wire.payload.is_object() => Event::Heartbeat,
        "sessionEnded" if wire.payload.is_object() => Event::End,
        _ => return Err("Unknown event".into()),
    };
    Ok(Envelope { sequence: wire.sequence, event })
}
