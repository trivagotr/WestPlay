//! Optional SDK expansion, independent of the main bridge ABI. Reader never creates or writes a mapping.
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
const MAGIC: u32 = 0x31574354;
fn name(pid: u32) -> String {
    format!("Local\\WestPlay.CoachTelemetry.v1.{pid}")
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct CoachTelemetry {
    pub pid: u32,
    pub supported: u32,
    pub counter: u64,
    pub known: u32,
    pub satisfaction: f32,
}
impl CoachTelemetry {
    pub fn value(v: Option<f32>) -> Option<f32> {
        v.filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
    pub fn satisfaction(&self) -> Option<f32> {
        if self.known & 1 != 0 { Self::value(Some(self.satisfaction)) } else { None }
    }
}
// SAFETY: contains only integer and float primitives, all byte patterns valid.
unsafe impl crate::Plain for CoachTelemetry {}
#[repr(C)]
struct Block {
    magic: AtomicU32,
    sample: SeqLock<CoachTelemetry>,
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
pub struct CoachWriter {
    mapping: Mapping,
    lock: Mutex<()>,
}
impl CoachWriter {
    pub fn open() -> windows::core::Result<Self> {
        Self::open_named(&name(std::process::id()))
    }
    fn open_named(name: &str) -> windows::core::Result<Self> {
        let mapping = Mapping::open(name, true)?;
        mapping.block().sample.write(&CoachTelemetry::default());
        mapping.block().magic.store(MAGIC, Ordering::Release);
        Ok(Self { mapping, lock: Mutex::new(()) })
    }
    pub fn publish(&self, sample: CoachTelemetry) {
        let _guard = self.lock.lock().unwrap_or_else(|e| e.into_inner());
        self.mapping.block().sample.write(&sample);
    }
}
pub struct CoachReader {
    mapping: Mapping,
}
impl CoachReader {
    pub fn open(pid: u32) -> windows::core::Result<Self> {
        Self::open_named(&name(pid))
    }
    fn open_named(name: &str) -> windows::core::Result<Self> {
        Ok(Self { mapping: Mapping::open(name, false)? })
    }
    pub fn read(&self) -> Option<CoachTelemetry> {
        let b = self.mapping.block();
        (b.magic.load(Ordering::Acquire) == MAGIC).then(|| b.sample.read()).flatten()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalized_values_distinguish_zero_unknown_and_invalid() {
        assert_eq!(CoachTelemetry::value(Some(0.0)), Some(0.0));
        assert_eq!(CoachTelemetry::value(Some(0.875)), Some(0.875));
        assert_eq!(CoachTelemetry::value(None), None);
        assert_eq!(CoachTelemetry::value(Some(f32::NAN)), None);
        assert_eq!(CoachTelemetry::value(Some(1.1)), None);
    }
    #[test]
    fn a_read_only_mapping_sees_the_writer_sample() {
        let name = format!("Local\\WestPlay.CoachTest.{}", std::process::id());
        let writer = CoachWriter::open_named(&name).unwrap();
        let reader = CoachReader::open_named(&name).unwrap();
        writer.publish(CoachTelemetry { pid: 77, counter: 1, supported: 1, known: 1, satisfaction: 0.875 });
        assert_eq!(reader.read().unwrap().satisfaction, 0.875);
        assert_eq!(reader.read().unwrap().pid, 77);
    }
}
