//! On-board flash erase/write.
//!
//! These operate directly on flash addresses and are inherently `unsafe`: a wrong
//! address or length can brick the device's stored data. Erase a sector before
//! writing it.

use crate::ffi;
use core::ffi::c_void;

/// Erase the flash sector containing `romp`. Returns the kernel result (0 = ok).
///
/// # Safety
/// `romp` must point into on-board flash; the whole containing sector is erased.
#[inline]
pub unsafe fn erase(romp: *mut c_void) -> i32 {
    ffi::pceFlashErase(romp)
}

/// Write `len` bytes from `memp` to flash at `romp`. Returns the kernel result.
///
/// # Safety
/// `romp` must point into on-board flash (in an already-erased region) and
/// `memp` must be readable for `len` bytes.
#[inline]
pub unsafe fn write(romp: *mut c_void, memp: *const c_void, len: i32) -> i32 {
    ffi::pceFlashWrite(romp, memp, len)
}
