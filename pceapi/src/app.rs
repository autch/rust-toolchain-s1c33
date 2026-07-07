//! Application lifecycle helpers.
//!
//! P/ECE apps are driven by four callbacks the kernel calls by symbol name
//! (`pceAppInit`, `pceAppProc`, `pceAppExit`, `pceAppNotify`); those are defined
//! by the application, not here. This module wraps the lifecycle *calls* an app
//! makes into the kernel, plus the notify constants.

use crate::ffi;
use core::ffi::CStr;

/// Set how often (in ms) the kernel calls `pceAppProc`. Returns the previous
/// period.
#[inline]
pub fn set_proc_period(period_ms: i32) -> i32 {
    unsafe { ffi::pceAppSetProcPeriod(period_ms) }
}

/// Ask the kernel to terminate this app with `exitcode`.
#[inline]
pub fn req_exit(exitcode: i32) {
    unsafe { ffi::pceAppReqExit(exitcode) }
}

/// Launch another app by filename, replacing the current one. Returns the kernel
/// result (on success it does not return here — the new app takes over).
#[inline]
pub fn exec_file(fname: &CStr, resv: i32) -> i32 {
    unsafe { ffi::pceAppExecFile(fname.as_ptr(), resv) }
}

/// Tell the kernel whether this app is "active" (affects power/scheduling).
#[inline]
pub fn set_active(active: bool) {
    let flag = if active { ffi::AAR_ACTIVE } else { ffi::AAR_NOACTIVE };
    unsafe { ffi::pceAppActiveResponse(flag) }
}

/// `pceAppNotify` event types (the `type` argument).
pub mod notify {
    use crate::ffi;

    pub const EXITREQ: i32 = ffi::APPNF_EXITREQ;
    pub const SMSTART: i32 = ffi::APPNF_SMSTART;
    pub const SMRESUME: i32 = ffi::APPNF_SMRESUME;
    pub const SMREQVBUF: i32 = ffi::APPNF_SMREQVBUF;
    pub const STANDBY: i32 = ffi::APPNF_STANDBY;
    pub const ALARM: i32 = ffi::APPNF_ALARM;
}

/// `pceAppNotify` return values (how the app responds to a notification).
pub mod response {
    use crate::ffi;

    pub const IGNORE: i32 = ffi::APPNR_IGNORE;
    pub const ACCEPT: i32 = ffi::APPNR_ACCEPT;
    pub const SUSPEND: i32 = ffi::APPNR_SUSPEND;
    pub const REJECT: i32 = ffi::APPNR_REJECT;
}
