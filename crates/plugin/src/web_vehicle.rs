//! Documented optional SDK channels for the web dashboard; no cabin UI changes.
use crate::scs::{
    ChannelCallback, SCS_RESULT_OK, SCS_U32_NIL, SCS_VALUE_TYPE_BOOL, SCS_VALUE_TYPE_FLOAT, ScsValue,
    TelemetryInitParamsV100,
};
use std::{
    ffi::{CString, c_char, c_void},
    sync::Mutex,
};
use westplay_shared::WebVehicle;
static SNAPSHOT: Mutex<WebVehicle> = Mutex::new(WebVehicle::empty());
const CHANNELS: &[&str] = &[
    "truck.wear.engine",
    "truck.wear.transmission",
    "truck.wear.cabin",
    "truck.wear.chassis",
    "truck.wear.wheels",
    "truck.fuel.warning",
    "truck.adblue.warning",
    "truck.brake.air.pressure.warning",
    "truck.brake.air.pressure.emergency",
    "truck.oil.pressure.warning",
    "truck.water.temperature.warning",
    "truck.battery.voltage.warning",
];
/// # Safety
/// Called only during SCS telemetry init with the SDK parameters.
pub unsafe fn register(params: &TelemetryInitParamsV100) {
    type Register = unsafe extern "system" fn(*const c_char, u32, u32, u32, ChannelCallback, *mut c_void) -> i32;
    // SAFETY: SDK function pointer and callback signatures match their header definitions.
    unsafe {
        let register: Register = std::mem::transmute(params.register_for_channel);
        for (index, name) in CHANNELS.iter().enumerate() {
            let name = CString::new(*name).unwrap();
            let kind = if index < 5 { SCS_VALUE_TYPE_FLOAT } else { SCS_VALUE_TYPE_BOOL };
            let result = register(name.as_ptr(), SCS_U32_NIL, kind, 0, on_channel, index as *mut c_void);
            crate::log!(
                "web telemetry: {} registration {}",
                name.to_string_lossy(),
                if result == SCS_RESULT_OK { "ok" } else { "unavailable" }
            );
        }
    }
}
// SAFETY: invoked only by the SDK with the type registered above; null means unavailable.
unsafe extern "system" fn on_channel(_name: *const c_char, _index: u32, value: *const ScsValue, context: *mut c_void) {
    let index = context as usize;
    let mut state = SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner());
    // SAFETY: SDK value lives through this callback and has the registered type.
    unsafe {
        if index < 5 {
            state.set_wear(index, value.as_ref().map(|v| v.float()));
        } else {
            state.set_warning(index - 5, value.as_ref().map(|v| v.bool()));
        }
    }
}
pub fn publish() {
    if let Some(bridge) = crate::bridge::get() {
        bridge.web_vehicle.write(&*SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner()));
    }
}

/// Tracks availability of existing cabin channels independently from their default values.
pub fn available(index: usize, known: bool) {
    SNAPSHOT.lock().unwrap_or_else(|e| e.into_inner()).set_known(index, known);
}
