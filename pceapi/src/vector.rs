//! Trap and kernel-service vector table (low-level).
//!
//! These install raw handlers into the CPU/kernel vector table. They are
//! inherently `unsafe`: a wrong index or a handler that violates the ABI corrupts
//! interrupt/exception dispatch. Handlers run in interrupt context.

use crate::ffi;

pub use crate::ffi::{PCEKSENT, PCETPENT};

/// Install a trap-vector handler at index `no`; returns the previous entry.
///
/// # Safety
/// `no` must be a valid trap index and `handler` a correct `extern "C"` handler
/// that does not unwind.
#[inline]
pub unsafe fn set_trap(no: i32, handler: PCETPENT) -> PCETPENT {
    ffi::pceVectorSetTrap(no, handler)
}

/// Install a kernel-service entry at index `no`; returns the previous entry.
///
/// # Safety
/// `no` must be a valid service index and `entry` a correct handler pointer.
#[inline]
pub unsafe fn set_ks(no: i32, entry: PCEKSENT) -> PCEKSENT {
    ffi::pceVectorSetKs(no, entry)
}
