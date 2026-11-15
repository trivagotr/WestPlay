//! Optional web-only instrumentation. Unknown channels remain unavailable.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct WebVehicle {
    wear_values: [f32; 5],
    wear_known: u32,
    core_known: u32,
    warning_values: u32,
    warning_known: u32,
}
impl WebVehicle {
    pub fn known(&self, index: usize) -> bool {
        index < 10 && self.core_known & (1 << index) != 0
    }
    pub fn set_known(&mut self, index: usize, known: bool) {
        if index < 10 {
            self.core_known &= !(1 << index);
            if known {
                self.core_known |= 1 << index;
            }
        }
    }
    pub const fn empty() -> Self {
        Self { wear_values: [0.0; 5], wear_known: 0, core_known: 0, warning_values: 0, warning_known: 0 }
    }
    pub fn wear(&self, index: usize) -> Option<f32> {
        (index < 5 && self.wear_known & (1 << index) != 0).then(|| self.wear_values[index])
    }
    pub fn set_wear(&mut self, index: usize, value: Option<f32>) {
        if index >= 5 {
            return;
        }
        self.wear_known &= !(1 << index);
        if let Some(v) = value.filter(|v| v.is_finite() && (0.0..=1.0).contains(v)) {
            self.wear_values[index] = v;
            self.wear_known |= 1 << index;
        }
    }
    pub fn warning(&self, index: usize) -> Option<bool> {
        (index < 8 && self.warning_known & (1 << index) != 0).then(|| self.warning_values & (1 << index) != 0)
    }
    pub fn set_warning(&mut self, index: usize, value: Option<bool>) {
        if index >= 8 {
            return;
        }
        self.warning_known &= !(1 << index);
        self.warning_values &= !(1 << index);
        if let Some(v) = value {
            self.warning_known |= 1 << index;
            if v {
                self.warning_values |= 1 << index;
            }
        }
    }
}
// SAFETY: only floats and integers, any byte pattern is valid (Plain's contract).
unsafe impl crate::Plain for WebVehicle {}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_is_not_zero_and_null_resets_known_channel() {
        let mut s = WebVehicle::default();
        assert!(!s.known(0));
        s.set_known(0, true);
        assert!(s.known(0));
        s.set_known(0, false);
        assert!(!s.known(0));
        assert_eq!(s.wear(0), None);
        s.set_wear(0, Some(0.25));
        assert_eq!(s.wear(0), Some(0.25));
        s.set_wear(0, None);
        assert_eq!(s.wear(0), None);
        s.set_wear(1, Some(f32::NAN));
        assert_eq!(s.wear(1), None);
        s.set_warning(0, Some(false));
        assert_eq!(s.warning(0), Some(false));
        s.set_warning(0, None);
        assert_eq!(s.warning(0), None);
    }
}
