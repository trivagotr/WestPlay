//! Optional readback of the final game texture. Map never waits for the GPU;
//! network/encoder work lives in westplay-remote, outside the host thread.
use crate::gpu::{Lock, SharedFrame};
use std::time::{Duration, Instant};
use westplay_remote::{Frame, RemoteControl};
use windows::{
    Win32::Graphics::Direct3D11::*,
    core::{Error, Result},
};

pub(crate) struct Readback {
    textures: [ID3D11Texture2D; 2],
    pending: [bool; 2],
    next: Instant,
    width: u32,
    height: u32,
    sequences: [u64; 2],
    submitted: u64,
    delivered: u64,
}
fn advance(sequence: u64, delivered: &mut u64) -> bool {
    if sequence <= *delivered {
        return false;
    }
    *delivered = sequence;
    true
}
impl Readback {
    pub fn new(context: &ID3D11DeviceContext, shared: &SharedFrame) -> Result<Self> {
        // SAFETY: the context owns a live D3D11 device; the output descriptors are initialized by D3D11.
        let device = unsafe { context.GetDevice()? };
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        // SAFETY: shared.texture is alive, and desc points to writable initialized storage.
        unsafe {
            shared.texture.GetDesc(&mut desc);
        }
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;
        let create = || -> Result<ID3D11Texture2D> {
            let mut texture = None;
            // SAFETY: desc describes a CPU-readable staging texture with no initial data.
            unsafe {
                device.CreateTexture2D(&desc, None, Some(&mut texture))?;
            }
            texture.ok_or_else(Error::empty)
        };
        Ok(Self {
            textures: [create()?, create()?],
            pending: [false; 2],
            next: Instant::now(),
            width: desc.Width,
            height: desc.Height,
            sequences: [0; 2],
            submitted: 0,
            delivered: 0,
        })
    }
    pub fn step(&mut self, context: &ID3D11DeviceContext, shared: &SharedFrame, remote: &RemoteControl) {
        // Readback completes outside the game texture's lock. A busy GPU simply
        // leaves the slot pending until a later host turn.
        for index in 0..2 {
            if !self.pending[index] {
                continue;
            }
            let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
            // SAFETY: staging texture is alive; map is read-only and explicitly nonblocking.
            if unsafe {
                context.Map(
                    &self.textures[index],
                    0,
                    D3D11_MAP_READ,
                    D3D11_MAP_FLAG_DO_NOT_WAIT.0 as u32,
                    Some(&mut mapped),
                )
            }
            .is_err()
            {
                continue;
            }
            let row = self.width as usize * 4;
            let mut bgra = vec![0u8; row * self.height as usize];
            if !mapped.pData.is_null() && mapped.RowPitch as usize >= row {
                for y in 0..self.height as usize {
                    // SAFETY: successful Map exposes height rows of at least RowPitch bytes. We copy only
                    // the actual width into an equally sized owned buffer, before Unmap below.
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            (mapped.pData as *const u8).add(y * mapped.RowPitch as usize),
                            bgra.as_mut_ptr().add(y * row),
                            row,
                        );
                    }
                }
                if advance(self.sequences[index], &mut self.delivered) {
                    remote.publish(Frame { bgra, width: self.width, height: self.height });
                }
            }
            // SAFETY: exactly the successfully mapped subresource above, no references retained.
            unsafe {
                context.Unmap(&self.textures[index], 0);
            }
            self.pending[index] = false;
        }
        let now = Instant::now();
        if now < self.next {
            return;
        }
        let Some(index) = self.pending.iter().position(|pending| !pending) else {
            return;
        };
        if shared.try_lock() != Lock::Held {
            return;
        }
        // SAFETY: texture sizes/formats match; shared is locked; destination is not mapped.
        unsafe {
            context.CopyResource(&self.textures[index], &shared.texture);
        }
        shared.unlock();
        self.pending[index] = true;
        self.submitted += 1;
        self.sequences[index] = self.submitted;
        self.next = now + Duration::from_micros(33_333);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn late_readbacks_never_replace_a_newer_delivered_frame() {
        let mut delivered = 0;
        assert!(advance(2, &mut delivered));
        assert!(!advance(1, &mut delivered));
        assert!(!advance(2, &mut delivered));
        assert_eq!(delivered, 2);
        assert!(advance(3, &mut delivered));
        assert_eq!(delivered, 3);
    }
}
