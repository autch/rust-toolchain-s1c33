//! Power / battery status.

use crate::ffi::{self, PCEPWRSTAT};

/// Read the current power status (source + battery voltage).
#[inline]
pub fn status() -> PCEPWRSTAT {
    let mut ps = PCEPWRSTAT::default();
    unsafe { ffi::pcePowerGetStatus(&mut ps) };
    ps
}

/// Running on battery (vs USB) power?
#[inline]
pub fn on_battery() -> bool {
    status().status != 0
}

/// Enable/disable power-event reporting (delivered via `pceAppNotify`).
#[inline]
pub fn set_report(on: bool) {
    let mode = if on { ffi::PWR_RPTON } else { ffi::PWR_RPTOFF };
    unsafe { ffi::pcePowerSetReport(mode) }
}

/// Enter standby; returns the kernel result code.
#[inline]
pub fn enter_standby(flag: i32) -> i32 {
    unsafe { ffi::pcePowerEnterStandby(flag) }
}
