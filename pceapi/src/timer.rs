//! Timers.

use crate::ffi;

/// Free-running tick counter.
#[inline]
pub fn count() -> u32 {
    unsafe { ffi::pceTimerGetCount() }
}

/// High-precision cycle counter (for timing measurements).
#[inline]
pub fn precision_count() -> u32 {
    unsafe { ffi::pceTimerGetPrecisionCount() }
}

/// Elapsed precision-counter delta from `start` to `end` (handles wraparound).
#[inline]
pub fn adjust_precision_count(start: u32, end: u32) -> u32 {
    unsafe { ffi::pceTimerAdjustPrecisionCount(start, end) }
}

/// Callback firing mode for [`set_callback`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum Mode {
    None = ffi::PCE_TT_NONE,
    OneShot = ffi::PCE_TT_ONESHOT,
    Periodic = ffi::PCE_TT_PERIODIC,
}

/// Install a timer callback on channel `ch`, firing after `time` (mode-dependent).
///
/// # Safety
/// `callback` runs in interrupt context: it must be `extern "C"`, must not unwind,
/// and may race with the rest of the program. Pass `None` to clear.
#[inline]
pub unsafe fn set_callback(ch: i32, mode: Mode, time: i32, callback: Option<unsafe extern "C" fn()>) {
    ffi::pceTimerSetCallback(ch, mode as i32, time, callback)
}
