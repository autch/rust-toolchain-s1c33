//! Debug monitor control.

use crate::ffi;

/// Set the debug-monitor mode.
#[inline]
pub fn set_mon(mode: i32) {
    unsafe { ffi::pceDebugSetMon(mode) }
}
