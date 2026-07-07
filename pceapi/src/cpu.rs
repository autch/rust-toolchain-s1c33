//! CPU clock speed.

use crate::ffi;

/// CPU clock speed for [`set_speed`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Speed {
    Normal = ffi::CPU_SPEED_NORMAL,
    Half = ffi::CPU_SPEED_HALF,
}

/// Set the CPU clock speed; returns the previous speed code.
#[inline]
pub fn set_speed(speed: Speed) -> i32 {
    unsafe { ffi::pceCPUSetSpeed(speed as i32) }
}
