//! Documented SDK channels only. Lane assist and Coaches schedule data are not exposed by this SDK.
use crate::scs::{
    ChannelCallback, ConfigurationEvent, SCS_RESULT_OK, SCS_U32_NIL, SCS_VALUE_TYPE_FLOAT, SCS_VALUE_TYPE_STRING,
    SCS_VALUE_TYPE_U32, ScsValue, TelemetryInitParamsV100,
};
use std::{
    ffi::{CStr, c_char, c_void},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use westplay_shared::dashboard::{DashboardTelemetry, DashboardWriter};
const NO_VALUE: u32 = 2;
static WRITER: OnceLock<Option<DashboardWriter>> = OnceLock::new();
static SNAPSHOT: Mutex<Option<DashboardTelemetry>> = Mutex::new(None);
static COUNTER: AtomicU64 = AtomicU64::new(0);
const CHANNELS: &[(&std::ffi::CStr, u32)] =
    &[(c"truck.fuel.consumption.average", SCS_VALUE_TYPE_FLOAT), (c"job.cargo.damage", SCS_VALUE_TYPE_FLOAT)];
/// # Safety
/// Called with the SDK registration parameters during telemetry initialization.
pub unsafe fn register(params: &TelemetryInitParamsV100) {
    WRITER.get_or_init(|| {
        DashboardWriter::open().map_err(|e| crate::log!("dashboard telemetry: mapping unavailable: {e}")).ok()
    });
    *SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner()) = Some(DashboardTelemetry::default());
    type Register = unsafe extern "system" fn(*const c_char, u32, u32, u32, ChannelCallback, *mut c_void) -> i32;
    // SAFETY: SDK registration signature and channel value types match the SDK definitions.
    let register: Register = unsafe { std::mem::transmute(params.register_for_channel) };
    for (index, (name, kind)) in CHANNELS.iter().enumerate() {
        // SAFETY: static channel name and callback live for the complete SDK session.
        let result = unsafe { register(name.as_ptr(), SCS_U32_NIL, *kind, NO_VALUE, on_value, index as *mut c_void) };
        crate::log!(
            "dashboard telemetry: {} registration {}",
            name.to_string_lossy(),
            if result == SCS_RESULT_OK { "ok" } else { "unavailable" }
        );
    }
}
// SAFETY: invoked by the SDK with the registered type or null (channel unavailable).
unsafe extern "system" fn on_value(_name: *const c_char, _index: u32, value: *const ScsValue, context: *mut c_void) {
    let index = context as usize;
    if index >= CHANNELS.len() {
        return;
    }
    // SAFETY: SDK value remains valid throughout the callback; type is verified before reading.
    let next = unsafe { value.as_ref() }.filter(|v| v.value_type == CHANNELS[index].1).map(|v| v.float() as f64);
    update_number(index, next);
}
/// Forward channels already registered by the core telemetry callback, including null availability.
pub fn update_number(index: usize, value: Option<f64>) {
    SNAPSHOT
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get_or_insert_with(DashboardTelemetry::default)
        .set_number(index, value);
}
pub fn publish() {
    let Some(Some(writer)) = WRITER.get() else { return };
    let mut sample = SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner()).unwrap_or_default();
    sample.pid = std::process::id();
    sample.counter = COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    writer.publish(sample);
}
#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn sdk(
        name: *const c_char,
        index: u32,
        kind: u32,
        flags: u32,
        callback: ChannelCallback,
        context: *mut c_void,
    ) -> i32 {
        assert_eq!(flags & NO_VALUE, NO_VALUE);
        // SAFETY: zero is valid for all primitive fields of the SDK union.
        let mut value: ScsValue = unsafe { std::mem::zeroed() };
        value.value_type = kind;
        // SAFETY: synchronous fixture callback supplies the registered primitive type, then unavailable.
        unsafe {
            callback(name, index, &value, context);
            callback(name, index, std::ptr::null(), context);
        }
        SCS_RESULT_OK
    }
    #[test]
    fn sdk_channel_loss_clears_values_and_unsupported_features_are_not_guessed() {
        // SAFETY: initialization structure consists only of pointers and integer fields.
        let mut params: TelemetryInitParamsV100 = unsafe { std::mem::zeroed() };
        params.register_for_channel = sdk as *const c_void;
        // SAFETY: fixture has precisely the SDK registration function signature.
        unsafe { register(&params) };
        let sample = SNAPSHOT.lock().unwrap().unwrap();
        for i in 0..5 {
            assert_eq!(sample.number(i), None);
        }
        assert_eq!(sample.boolean(0), None);
        assert_eq!(sample.boolean(1), None);
        assert_eq!(sample.destination(), None);
        // SAFETY: SDK attribute fixtures contain only primitive pointers and integer fields.
        let mut attrs: [crate::scs::ScsNamedValue; 3] = unsafe { std::mem::zeroed() };
        attrs[0].name = c"destination.city".as_ptr();
        attrs[0].value.value_type = SCS_VALUE_TYPE_STRING;
        attrs[1].name = c"delivery.time".as_ptr();
        attrs[1].value.value_type = SCS_VALUE_TYPE_U32;
        // SAFETY: SDK ScsValue union begins at byte8 and has room/alignment for both fixtures.
        unsafe {
            ((&mut attrs[0].value as *mut ScsValue as *mut u8).add(8) as *mut *const c_char).write(c"Ankara".as_ptr());
            ((&mut attrs[1].value as *mut ScsValue as *mut u8).add(8) as *mut u32).write(1441);
        }
        let job = ConfigurationEvent { id: c"job".as_ptr(), attributes: attrs.as_ptr() };
        // SAFETY: fixture includes a terminating null-name entry and remains live.
        unsafe { configuration(&job) };
        let sample = SNAPSHOT.lock().unwrap().unwrap();
        assert_eq!(sample.boolean(1), Some(true));
        assert_eq!(sample.destination(), Some("Ankara"));
        assert_eq!(sample.number(5), Some(1441.0));
        let bus = ConfigurationEvent { id: c"bus_job".as_ptr(), attributes: attrs.as_ptr() };
        // SAFETY: same live terminated fixture; undocumented bus attrs must not be interpreted.
        unsafe { configuration(&bus) };
        assert_eq!(SNAPSHOT.lock().unwrap().unwrap().boolean(2), Some(true));
        let ended = ConfigurationEvent { id: c"job".as_ptr(), attributes: std::ptr::null() };
        // SAFETY: null attributes represent an SDK job-end configuration.
        unsafe { configuration(&ended) };
        let sample = SNAPSHOT.lock().unwrap().unwrap();
        assert_eq!(sample.boolean(1), Some(false));
        assert_eq!(sample.destination(), None);
        assert_eq!(sample.number(5), None);
        assert_eq!(sample.number(1), None);
        let ended = ConfigurationEvent { id: c"bus_job".as_ptr(), attributes: std::ptr::null() };
        // SAFETY: null attributes represent an SDK bus-job-end configuration.
        unsafe { configuration(&ended) };
        assert_eq!(SNAPSHOT.lock().unwrap().unwrap().boolean(2), Some(false));
    }
}

static BUS_SCHEMA: Mutex<String> = Mutex::new(String::new());
/// # Safety
/// SDK-owned configuration and attributes remain alive through this callback.
pub unsafe fn configuration(config: &ConfigurationEvent) {
    if config.id.is_null() {
        return;
    }
    // SAFETY: SDK provides a NUL-terminated configuration identifier.
    let id = unsafe { CStr::from_ptr(config.id) }.to_bytes();
    if id != b"job" && id != b"bus_job" {
        return;
    }
    // SAFETY: attributes belong to this live SDK configuration callback.
    let attrs: Vec<_> = unsafe { config.attributes() }.take(128).collect();
    let mut state = SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner());
    let sample = state.get_or_insert_with(DashboardTelemetry::default);
    if id == b"bus_job" {
        sample.set_bool(2, Some(!attrs.is_empty()));
        // Discover names/types only, never infer the semantics of undocumented attributes.
        let schema = attrs
            .iter()
            .take(32)
            .map(|a| {
                // SAFETY: iterator excludes the null-name terminator; SDK name lives through callback.
                let name = unsafe { CStr::from_ptr(a.name) }.to_string_lossy().chars().take(64).collect::<String>();
                format!("{name}:{}", a.value.value_type)
            })
            .collect::<Vec<_>>()
            .join(", ");
        let schema = if schema.is_empty() { "empty".to_owned() } else { schema };
        let mut last = BUS_SCHEMA.lock().unwrap_or_else(|e| e.into_inner());
        if *last != schema {
            crate::log!("dashboard bus_job SDK attribute names/types: {schema}");
            *last = schema;
        }
        return;
    }
    sample.set_bool(1, Some(!attrs.is_empty()));
    sample.set_destination(None);
    sample.set_number(5, None);
    if attrs.is_empty() {
        sample.set_number(1, None);
    }
    for attr in attrs {
        // SAFETY: SDK name and typed value live through this callback.
        match unsafe { CStr::from_ptr(attr.name) }.to_bytes() {
            b"destination.city" if attr.value.value_type == SCS_VALUE_TYPE_STRING => {
                sample.set_destination(attr.value.string().as_deref())
            }
            b"delivery.time" if attr.value.value_type == SCS_VALUE_TYPE_U32 => {
                sample.set_number(5, Some(attr.value.u32() as f64))
            }
            _ => {}
        }
    }
}
