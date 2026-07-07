//! Raw FFI: `extern "C"` declarations matching `sdk/include/piece.h`, plus the
//! kernel API constants. All symbols are provided by `libpceapi.a` (they are not
//! declared in a header for the Rust side — these signatures are the contract).
//!
//! Signatures use `core::ffi` types: on `s1c33-none-piece`, `int`/`long` are
//! 32-bit, `unsigned long` is `c_ulong` (u32), `char` is signed (`c_char` = i8).
//!
//! Only the core subset is declared here. Add more as needed from piece.h.

#![allow(non_snake_case)]

use core::ffi::{c_char, c_int, c_uchar, c_ulong, c_ushort, c_void};

extern "C" {
    // ---- Pad (input) ----
    pub fn pcePadGet() -> c_ulong;
    pub fn pcePadGetDirect() -> c_ulong;
    pub fn pcePadSetTrigMode(mode: c_int);

    // ---- LCD ----
    pub fn pceLCDDispStart();
    pub fn pceLCDDispStop();
    pub fn pceLCDTrans();
    pub fn pceLCDTransDirect(lcd_direct: *const c_uchar);
    pub fn pceLCDTransRange(xs: c_int, ys: c_int, xe: c_int, ye: c_int);
    pub fn pceLCDSetBuffer(pbuff: *mut c_uchar) -> *mut c_uchar;
    pub fn pceLCDSetOrientation(dir: c_int) -> c_int;
    pub fn pceLCDSetBright(bright: c_int) -> c_int;

    // ---- Font ----
    pub fn pceFontGetAdrs(code: c_ushort) -> *const c_uchar;
    pub fn pceFontPut(x: c_int, y: c_int, code: c_ushort) -> c_ushort;
    pub fn pceFontSetType(ty: c_int);
    pub fn pceFontSetTxColor(color: c_int);
    pub fn pceFontSetBkColor(color: c_int);
    pub fn pceFontSetPos(x: c_int, y: c_int);
    pub fn pceFontPutStr(pstr: *const c_char) -> c_int;
    /// Variadic — exercises the s1c33 c-variadic ABI. Declared for completeness;
    /// calling it is how that ABI path gets tested.
    pub fn pceFontPrintf(format: *const c_char, ...) -> c_int;

    // ---- App lifecycle ----
    pub fn pceAppSetProcPeriod(period: c_int) -> c_int;
    pub fn pceAppReqExit(exitcode: c_int);

    // ---- CPU ----
    pub fn pceCPUSetSpeed(no: c_int) -> c_int;

    // ---- Heap ----
    pub fn pceHeapAlloc(size: c_ulong) -> *mut c_void;
    pub fn pceHeapFree(memp: *mut c_void) -> c_int;
    pub fn pceHeapRealloc(memp: *mut c_void, size: c_ulong) -> c_int;
    pub fn pceHeapGetMaxFreeSize() -> c_int;

    // ---- Timer / CRC (struct-free subset) ----
    pub fn pceTimerGetCount() -> c_ulong;
    pub fn pceTimerGetPrecisionCount() -> c_ulong;
    pub fn pceCRC32(ptr: *const c_void, len: c_ulong) -> c_ulong;
}

// ---- Constants (from piece.h #defines) ----

// Pad: currently-held bit masks. Trigger (newly-pressed) masks are these << 8.
pub const PAD_RI: c_ulong = 0x01;
pub const PAD_LF: c_ulong = 0x02;
pub const PAD_DN: c_ulong = 0x04;
pub const PAD_UP: c_ulong = 0x08;
pub const PAD_B: c_ulong = 0x10;
pub const PAD_A: c_ulong = 0x20;
pub const PAD_D: c_ulong = 0x40; // SELECT
pub const PAD_C: c_ulong = 0x80; // START

// pcePadSetTrigMode
pub const PP_MODE_SINGLE: c_int = 0;
pub const PP_MODE_REPEAT: c_int = 1;

// pceCPUSetSpeed
pub const CPU_SPEED_NORMAL: c_int = 0;
pub const CPU_SPEED_HALF: c_int = 1;

// pceAppNotify types (APPNF_*) and responses (APPNR_*)
pub const APPNF_EXITREQ: c_int = 1;
pub const APPNF_SMSTART: c_int = 2;
pub const APPNF_SMRESUME: c_int = 3;
pub const APPNF_SMREQVBUF: c_int = 4;
pub const APPNF_STANDBY: c_int = 5;
pub const APPNF_ALARM: c_int = 6;

pub const APPNR_IGNORE: c_int = 0;
pub const APPNR_ACCEPT: c_int = 1;
pub const APPNR_SUSPEND: c_int = 2;
pub const APPNR_REJECT: c_int = 3;
