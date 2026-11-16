//! Coaches' engine SDK channel (found in 1.62.3.3); no internal game calls or memory offsets.
use crate::scs::{
    ChannelCallback, SCS_RESULT_OK, SCS_U32_NIL, SCS_VALUE_TYPE_FLOAT, ScsValue, TelemetryInitParamsV100,
};
use std::{
    ffi::{c_char, c_void},
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU64, Ordering},
    },
};
use westplay_shared::coach::{CoachTelemetry, CoachWriter};
// SDK SCS_TELEMETRY_CHANNEL_FLAG_no_value: receive null when a job/channel disappears.
const NO_VALUE: u32 = 0x2;
static WRITER: OnceLock<Option<CoachWriter>> = OnceLock::new();
static COUNTER: AtomicU64 = AtomicU64::new(0);
struct Snapshot {
    supported: bool,
    value: Option<f32>,
}
static SNAPSHOT: Mutex<Snapshot> = Mutex::new(Snapshot { supported: false, value: None });
/// # Safety
/// SDK init parameters must be those passed to this plugin on its registration thread.
pub unsafe fn register(params: &TelemetryInitParamsV100) {
    WRITER.get_or_init(|| match CoachWriter::open() {
        Ok(w) => Some(w),
        Err(e) => {
            crate::log!("coach telemetry: mapping unavailable: {e}");
            None
        }
    });
    type Register = unsafe extern "system" fn(*const c_char, u32, u32, u32, ChannelCallback, *mut c_void) -> i32;
    // SAFETY: matching SDK registration function signature, callback type and static channel string.
    let result = unsafe {
        let register: Register = std::mem::transmute(params.register_for_channel);
        register(
            c"bus_job.average_satisfaction".as_ptr(),
            SCS_U32_NIL,
            SCS_VALUE_TYPE_FLOAT,
            NO_VALUE,
            on_value,
            std::ptr::null_mut(),
        )
    };
    SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner()).supported = result == SCS_RESULT_OK;
    crate::log!("coach telemetry: bus_job.average_satisfaction float registration result {result}");
}
// SAFETY: game calls this with the float value registered above, or null when unavailable.
unsafe extern "system" fn on_value(_name: *const c_char, _index: u32, value: *const ScsValue, _context: *mut c_void) {
    // SAFETY: SDK-owned float value lives through this callback; null is allowed.
    let next = CoachTelemetry::value(unsafe { value.as_ref().map(|v| v.float()) });
    let mut state = SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner());
    if state.value.is_none() != next.is_none()
        || state.value.zip(next).is_some_and(|(old, new)| (old - new).abs() >= 0.005)
    {
        if let Some(v) = next {
            crate::log!("coach telemetry: average satisfaction {:.2}%", v * 100.0)
        } else {
            crate::log!("coach telemetry: average satisfaction unavailable")
        }
    }
    state.value = next;
}
pub fn publish() {
    let Some(Some(writer)) = WRITER.get() else { return };
    let state = SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner());
    writer.publish(CoachTelemetry {
        pid: std::process::id(),
        supported: u32::from(state.supported),
        counter: COUNTER.fetch_add(1, Ordering::Relaxed) + 1,
        known: u32::from(state.value.is_some()),
        satisfaction: state.value.unwrap_or(0.0),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    unsafe extern "system" fn simulate_sdk(
        name: *const c_char,
        index: u32,
        _kind: u32,
        flags: u32,
        callback: ChannelCallback,
        context: *mut c_void,
    ) -> i32 {
        // SAFETY: ScsValue is a plain primitive union with valid zero initialization; set registered type.
        let mut value: ScsValue = unsafe { std::mem::zeroed() };
        value.value_type = SCS_VALUE_TYPE_FLOAT;
        // SAFETY: fixture callback runs synchronously with live fixture data, matching SDK contract.
        unsafe {
            callback(name, index, &value, context);
            if flags & 2 != 0 {
                callback(name, index, std::ptr::null(), context);
            }
        }
        SCS_RESULT_OK
    }
    #[test]
    fn ending_the_bus_job_invalidates_the_previous_satisfaction() {
        // SAFETY: init params hold raw pointers/integers only, all may be null in this minimal fixture.
        let mut params: TelemetryInitParamsV100 = unsafe { std::mem::zeroed() };
        params.register_for_channel = simulate_sdk as *const c_void;
        // SAFETY: the fixture implements precisely the SDK register/callback contract used here.
        unsafe { register(&params) };
        assert_eq!(SNAPSHOT.lock().unwrap().value, None, "SDK unavailable callbacks must be requested");
    }
}
