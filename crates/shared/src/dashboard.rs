//! Optional SDK samples for the web dashboard, without extending the main bridge ABI.
use crate::SeqLock;
use std::sync::{
    Mutex,
    atomic::{AtomicU32, Ordering},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE},
        System::Memory::{
            CreateFileMappingW, FILE_MAP_ALL_ACCESS, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
            OpenFileMappingW, PAGE_READWRITE, UnmapViewOfFile,
        },
    },
    core::HSTRING,
};
const MAGIC: u32 = 0x31445754;
fn name(pid: u32) -> String {
    format!("Local\\WestPlay.DashboardTelemetry.v1.{pid}")
}

/// Numeric slots: fuel L/km, cargo damage, game time minutes, speed m/s, cruise m/s, cargo deadline minutes.
/// Bool slots: reserved for actual lane assist and active cargo / bus jobs; no inferred state.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct DashboardTelemetry {
    pub pid: u32,
    pub counter: u64,
    known_numbers: u32,
    numbers: [f64; 6],
    known_bools: u32,
    bools: u32,
    destination_length: u32,
    destination: [u8; 255],
}
impl Default for DashboardTelemetry {
    fn default() -> Self {
        Self {
            pid: 0,
            counter: 0,
            known_numbers: 0,
            numbers: [0.0; 6],
            known_bools: 0,
            bools: 0,
            destination_length: 0,
            destination: [0; 255],
        }
    }
}
impl DashboardTelemetry {
    pub fn number(&self, index: usize) -> Option<f64> {
        (index < self.numbers.len() && self.known_numbers & (1 << index) != 0)
            .then(|| self.numbers[index])
            .filter(|v| v.is_finite())
    }
    pub fn set_number(&mut self, index: usize, value: Option<f64>) {
        if index >= self.numbers.len() {
            return;
        }
        self.known_numbers &= !(1 << index);
        if let Some(value) = value.filter(|v| v.is_finite()) {
            self.numbers[index] = value;
            self.known_numbers |= 1 << index;
        }
    }
    pub fn boolean(&self, index: usize) -> Option<bool> {
        (index < 3 && self.known_bools & (1 << index) != 0).then(|| self.bools & (1 << index) != 0)
    }
    pub fn set_bool(&mut self, index: usize, value: Option<bool>) {
        if index >= 3 {
            return;
        }
        self.known_bools &= !(1 << index);
        self.bools &= !(1 << index);
        if let Some(value) = value {
            self.known_bools |= 1 << index;
            if value {
                self.bools |= 1 << index;
            }
        }
    }
    pub fn set_destination(&mut self, value: Option<&str>) {
        self.destination_length = 0;
        if let Some(value) = value {
            let mut length = value.len().min(self.destination.len());
            while !value.is_char_boundary(length) {
                length -= 1;
            }
            self.destination[..length].copy_from_slice(&value.as_bytes()[..length]);
            self.destination_length = length as u32;
        }
    }
    pub fn destination(&self) -> Option<&str> {
        let length = self.destination_length as usize;
        if length == 0 || length > self.destination.len() {
            None
        } else {
            std::str::from_utf8(&self.destination[..length]).ok()
        }
    }
}
// SAFETY: contains only integer and float primitives, all byte patterns valid.
unsafe impl crate::Plain for DashboardTelemetry {}
#[repr(C)]
struct Block {
    magic: AtomicU32,
    sample: SeqLock<DashboardTelemetry>,
}
struct Mapping {
    handle: HANDLE,
    view: MEMORY_MAPPED_VIEW_ADDRESS,
}
// SAFETY: mapped data is accessed only through atomics and SeqLock; handle/view stay alive with self.
unsafe impl Send for Mapping {}
// SAFETY: as Send; writer wrapper serializes all writes.
unsafe impl Sync for Mapping {}
impl Mapping {
    fn open(name: &str, writer: bool) -> windows::core::Result<Self> {
        // SAFETY: size is exactly Block, zero initialization is valid; handles are closed on mapping failure.
        unsafe {
            let handle = if writer {
                CreateFileMappingW(
                    INVALID_HANDLE_VALUE,
                    None,
                    PAGE_READWRITE,
                    0,
                    size_of::<Block>() as u32,
                    &HSTRING::from(name),
                )?
            } else {
                OpenFileMappingW(FILE_MAP_READ.0, false, &HSTRING::from(name))?
            };
            let view = MapViewOfFile(
                handle,
                if writer { FILE_MAP_ALL_ACCESS } else { FILE_MAP_READ },
                0,
                0,
                size_of::<Block>(),
            );
            if view.Value.is_null() {
                let error = windows::core::Error::from_thread();
                let _ = CloseHandle(handle);
                return Err(error);
            }
            Ok(Self { handle, view })
        }
    }
    fn block(&self) -> &Block {
        // SAFETY: mapped exactly Block; borrow cannot outlive mapping, fields atomic/plain only.
        unsafe { &*(self.view.Value as *const Block) }
    }
}
impl Drop for Mapping {
    fn drop(&mut self) {
        // SAFETY: only this object owns the view and handle; each is released once.
        unsafe {
            let _ = UnmapViewOfFile(self.view);
            let _ = CloseHandle(self.handle);
        }
    }
}
pub struct DashboardWriter {
    mapping: Mapping,
    lock: Mutex<()>,
}
impl DashboardWriter {
    pub fn open() -> windows::core::Result<Self> {
        Self::open_named(&name(std::process::id()))
    }
    fn open_named(name: &str) -> windows::core::Result<Self> {
        let mapping = Mapping::open(name, true)?;
        mapping.block().sample.write(&DashboardTelemetry::default());
        mapping.block().magic.store(MAGIC, Ordering::Release);
        Ok(Self { mapping, lock: Mutex::new(()) })
    }
    pub fn publish(&self, sample: DashboardTelemetry) {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.mapping.block().sample.write(&sample);
    }
}
pub struct DashboardReader {
    mapping: Mapping,
}
impl DashboardReader {
    pub fn open(pid: u32) -> windows::core::Result<Self> {
        Self::open_named(&name(pid))
    }
    fn open_named(name: &str) -> windows::core::Result<Self> {
        Ok(Self { mapping: Mapping::open(name, false)? })
    }
    pub fn read(&self) -> Option<DashboardTelemetry> {
        let b = self.mapping.block();
        (b.magic.load(Ordering::Acquire) == MAGIC).then(|| b.sample.read()).flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reader_observes_per_process_sidecar_without_creating_main_bridge() {
        let name = format!("Local\\WestPlay.DashboardTest.{}", std::process::id());
        let writer = DashboardWriter::open_named(&name).unwrap();
        let reader = DashboardReader::open_named(&name).unwrap();
        let mut sample = DashboardTelemetry::default();
        sample.pid = 77;
        sample.counter = 9;
        sample.set_number(0, Some(0.345));
        sample.set_number(2, Some(u32::MAX as f64));
        writer.publish(sample);
        let got = reader.read().unwrap();
        assert_eq!(got.pid, 77);
        assert_eq!(got.counter, 9);
        assert_eq!(got.number(0), Some(0.345));
        assert_eq!(got.number(2), Some(u32::MAX as f64));
    }
    #[test]
    fn values_preserve_zero_and_reject_invalid_data() {
        let mut sample = DashboardTelemetry::default();
        assert_eq!(sample.number(0), None);
        sample.set_number(0, Some(0.0));
        assert_eq!(sample.number(0), Some(0.0));
        sample.set_number(0, Some(f64::NAN));
        assert_eq!(sample.number(0), None);
        sample.set_bool(0, Some(false));
        assert_eq!(sample.boolean(0), Some(false));
        sample.set_bool(0, None);
        assert_eq!(sample.boolean(0), None);
    }
    #[test]
    fn strings_survive_utf8_boundary_without_partial_character() {
        let mut sample = DashboardTelemetry::default();
        sample.set_destination(Some(&"ğ".repeat(200)));
        assert_eq!(sample.destination().unwrap().chars().count(), 127);
        sample.set_destination(None);
        assert_eq!(sample.destination(), None);
    }
}
