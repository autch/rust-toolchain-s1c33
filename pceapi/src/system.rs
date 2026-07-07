//! System / hardware information.

use crate::ffi::{self, SYSTEMINFO};

/// Kernel system info (SRAM/PFFS bounds, clock, BIOS/HW versions).
///
/// Returns a reference to a kernel-owned static, valid for the app's lifetime.
#[inline]
pub fn info() -> &'static SYSTEMINFO {
    // SAFETY: pceSystemGetInfo returns a pointer to a static kernel structure.
    unsafe { &*ffi::pceSystemGetInfo() }
}
