//! Real-time clock (date/time) and alarm.

use crate::ffi::{self, PCEALMTIME, PCETIME};

/// Read the current date/time.
#[inline]
pub fn get() -> PCETIME {
    let mut t = PCETIME::default();
    unsafe { ffi::pceTimeGet(&mut t) };
    t
}

/// Set the date/time.
#[inline]
pub fn set(t: &PCETIME) {
    unsafe { ffi::pceTimeSet(t) }
}

/// Read the alarm setting; returns the kernel result code and the alarm.
#[inline]
pub fn get_alarm() -> (i32, PCEALMTIME) {
    let mut a = PCEALMTIME::default();
    let r = unsafe { ffi::pceTimeGetAlarm(&mut a) };
    (r, a)
}

/// Set the alarm; returns the kernel result code.
#[inline]
pub fn set_alarm(a: &PCEALMTIME) -> i32 {
    unsafe { ffi::pceTimeSetAlarm(a) }
}

/// Alarm modes for [`PCEALMTIME::mode`].
pub mod alarm {
    use crate::ffi;
    pub const STOP: u32 = ffi::ALM_STOP;
    pub const EVERY_HOUR: u32 = ffi::ALM_EVERYHOUR;
    pub const EVERY_DAY: u32 = ffi::ALM_EVERYDAY;
    pub const ONESHOT: u32 = ffi::ALM_ONESHOT;
}
