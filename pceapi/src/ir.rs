//! Infrared (IrDA) communication.
//!
//! Transfers are asynchronous: the kernel keeps reading/writing the buffer after
//! the `start_*` call returns, so the buffer must stay valid until the transfer
//! completes — hence the `start_*` functions are `unsafe`. Any callbacks run in
//! interrupt context (`extern "C"`, no unwinding).

use crate::ffi;
use core::ffi::{c_int, c_ushort};

/// Begin receiving `len` bytes into `buf`.
///
/// # Safety
/// The kernel writes `buf` asynchronously; it must remain valid for `len` bytes
/// until the transfer finishes.
#[inline]
pub unsafe fn start_rx(buf: *mut u8, len: i32) {
    ffi::pceIRStartRx(buf, len)
}

/// Begin transmitting `len` bytes from `buf`.
///
/// # Safety
/// The kernel reads `buf` asynchronously; it must remain valid for `len` bytes
/// until the transfer finishes.
#[inline]
pub unsafe fn start_tx(buf: *const u8, len: i32) {
    ffi::pceIRStartTx(buf, len)
}

/// [`start_rx`] with a completion callback (given the received length).
///
/// # Safety
/// As [`start_rx`], and `callback` runs in interrupt context.
#[inline]
pub unsafe fn start_rx_ex(
    buf: *mut u8,
    len: i32,
    mode: i32,
    callback: Option<unsafe extern "C" fn(rlen: c_int) -> c_int>,
) {
    ffi::pceIRStartRxEx(buf, len, mode, callback)
}

/// [`start_tx`] with a completion callback.
///
/// # Safety
/// As [`start_tx`], and `callback` runs in interrupt context.
#[inline]
pub unsafe fn start_tx_ex(
    buf: *const u8,
    len: i32,
    mode: i32,
    callback: Option<unsafe extern "C" fn() -> c_int>,
) {
    ffi::pceIRStartTxEx(buf, len, mode, callback)
}

/// Raw-pulse receive; `rxproc` is called per pulse edge with `(flag, time)`.
///
/// # Safety
/// `rxproc` runs in interrupt context.
#[inline]
pub unsafe fn start_rx_pulse(
    mode: i32,
    rxproc: Option<unsafe extern "C" fn(flag: c_int, time: c_ushort)>,
    timeout: i32,
) {
    ffi::pceIRStartRxPulse(mode, rxproc, timeout)
}

/// Raw-pulse transmit; `txproc` supplies each pulse.
///
/// # Safety
/// `txproc` runs in interrupt context.
#[inline]
pub unsafe fn start_tx_pulse(mode: i32, txproc: Option<unsafe extern "C" fn(flag: c_int) -> c_int>) {
    ffi::pceIRStartTxPulse(mode, txproc)
}

/// Stop any IrDA transfer in progress.
#[inline]
pub fn stop() {
    unsafe { ffi::pceIRStop() }
}

/// Current IrDA status.
#[inline]
pub fn get_stat() -> i32 {
    unsafe { ffi::pceIRGetStat() }
}
