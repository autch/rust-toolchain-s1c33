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

/// `MAXFILENAME` from piece.h (filename buffers are `MAXFILENAME + 1`).
pub const MAXFILENAME: usize = 26;

// ---- Types (from piece.h; #[repr(C)], field offsets match the C layout) ----

/// Trap-vector entry: `void (*)(void)`.
pub type PCETPENT = Option<unsafe extern "C" fn()>;
/// Kernel-service entry: `void *`.
pub type PCEKSENT = *mut c_void;

/// `PCEWAVEINFO` — a queued wave (audio) buffer descriptor.
#[repr(C)]
pub struct PCEWAVEINFO {
    /// status (kernel-updated; read/write with volatile ops)
    pub stat: c_uchar, // 0
    pub type_: c_uchar,                                       // 1
    pub resv: c_ushort,                                       // 2
    pub p_data: *const c_void,                                // 4
    pub len: c_ulong,                                         // 8 (samples)
    pub next: *mut PCEWAVEINFO,                               // 12
    pub pf_end_proc: Option<unsafe extern "C" fn(*mut PCEWAVEINFO)>, // 16
}

/// `MEMBLK` — a memory block (base + length).
#[repr(C)]
pub struct MEMBLK {
    pub top: *mut c_uchar,
    pub len: c_ulong,
}

impl Default for MEMBLK {
    fn default() -> Self {
        MEMBLK { top: core::ptr::null_mut(), len: 0 }
    }
}

/// `FILEINFO` — a PFFS directory entry (for find).
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct FILEINFO {
    pub filename: [c_char; MAXFILENAME + 1],
    pub attr: c_uchar,
    pub length: c_ulong,
    pub adrs: c_ulong,
    pub works: [c_uchar; 16],
}

/// `FILEACC` — an open-file accessor.
#[repr(C)]
pub struct FILEACC {
    pub valid: c_ushort, // 0
    pub resv2: c_uchar,  // 2
    pub resv3: c_uchar,  // 3
    pub aptr: *const c_uchar, // 4
    pub fsize: c_ulong,  // 8
    pub chain: c_ushort, // 12
    pub bpos: c_ushort,  // 14
}

impl Default for FILEACC {
    /// A zeroed accessor to hand to `pceFileOpen`, which fills it in.
    fn default() -> Self {
        FILEACC {
            valid: 0,
            resv2: 0,
            resv3: 0,
            aptr: core::ptr::null(),
            fsize: 0,
            chain: 0,
            bpos: 0,
        }
    }
}

/// `PCETIME` — real-time-clock date/time.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PCETIME {
    pub yy: c_ushort, // year
    pub mm: c_uchar,  // month
    pub dd: c_uchar,  // day
    pub hh: c_uchar,  // hour
    pub mi: c_uchar,  // minute
    pub ss: c_uchar,  // second
    pub s100: c_uchar, // 1/100 s
}

/// `PCEALMTIME` — alarm mode + time.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PCEALMTIME {
    pub mode: c_ulong,
    pub time: PCETIME,
}

/// `PCEPWRSTAT` — power/battery status.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct PCEPWRSTAT {
    pub status: c_uchar,  // 0: 0=USB power, 1=battery
    pub resv: c_uchar,    // 1
    pub battvol: c_ushort, // 2: battery voltage (mV)
}

/// `USBCOMINFO` — USB-COM device signature.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct USBCOMINFO {
    pub signature: [c_uchar; 16],
}

/// `SYSTEMINFO` — system/hardware information (from `pceSystemGetInfo`).
#[repr(C)]
pub struct SYSTEMINFO {
    pub size: c_ushort,       // 0
    pub hard_ver: c_ushort,   // 2
    pub bios_ver: c_ushort,   // 4
    pub bios_date: c_ushort,  // 6 (YY:MM:DD packed)
    pub sys_clock: c_ulong,   // 8 (Hz)
    pub vdde_voltage: c_ushort, // 12 (mV)
    pub resv1: c_ushort,      // 14
    pub sram_top: *mut c_uchar, // 16
    pub sram_end: *mut c_uchar, // 20
    pub pffs_top: *mut c_uchar, // 24
    pub pffs_end: *mut c_uchar, // 28
}

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

    // ---- Timer ----
    pub fn pceTimerGetCount() -> c_ulong;
    pub fn pceTimerGetPrecisionCount() -> c_ulong;
    pub fn pceTimerAdjustPrecisionCount(st: c_ulong, ed: c_ulong) -> c_ulong;
    pub fn pceTimerSetCallback(
        ch: c_int,
        ty: c_int,
        time: c_int,
        callback: Option<unsafe extern "C" fn()>,
    );
    pub fn pceTimerSetContextSwitcher(
        p: Option<unsafe extern "C" fn(nowsp: c_ulong, flag: c_int) -> c_ulong>,
    );

    // ---- CRC ----
    pub fn pceCRC32(ptr: *const c_void, len: c_ulong) -> c_ulong;

    // ---- Vector (trap / kernel-service entries) ----
    pub fn pceVectorSetTrap(no: c_int, adrs: PCETPENT) -> PCETPENT;
    pub fn pceVectorSetKs(no: c_int, adrs: PCEKSENT) -> PCEKSENT;

    // ---- App (extended) ----
    pub fn pceAppExecFile(fname: *const c_char, resv: c_int) -> c_int;
    pub fn pceAppGetHeap(pmb: *mut MEMBLK) -> c_int;
    pub fn pceAppActiveResponse(flag: c_int);

    // ---- Flash ----
    pub fn pceFlashErase(romp: *mut c_void) -> c_int;
    pub fn pceFlashWrite(romp: *mut c_void, memp: *const c_void, len: c_int) -> c_int;

    // ---- Wave (audio) ----
    pub fn pceWaveCheckBuffs(ch: c_int) -> c_int;
    pub fn pceWaveDataOut(ch: c_int, pwave: *mut PCEWAVEINFO) -> c_int;
    pub fn pceWaveAbort(ch: c_int) -> c_int;
    pub fn pceWaveSetChAtt(ch: c_int, att: c_int) -> c_int;
    pub fn pceWaveSetMasterAtt(att: c_int) -> c_int;
    pub fn pceWaveStop(hard: c_int);

    // ---- File (find / load / sector access / APF) ----
    pub fn pceFileFindOpen(pfi: *mut FILEINFO) -> c_int;
    pub fn pceFileFindNext(pfi: *mut FILEINFO) -> c_int;
    pub fn pceFileFindClose(pfi: *mut FILEINFO) -> c_int;
    pub fn pceFileLoad(fname: *const c_char, ptr: *mut c_void) -> c_int;
    pub fn pceFileOpen(pfa: *mut FILEACC, fname: *const c_char, mode: c_int) -> c_int;
    pub fn pceFileReadSct(pfa: *mut FILEACC, ptr: *mut c_void, sct: c_int, len: c_int) -> c_int;
    pub fn pceFileWriteSct(pfa: *mut FILEACC, ptr: *const c_void, sct: c_int, len: c_int) -> c_int;
    pub fn pceFileClose(pfa: *mut FILEACC) -> c_int;
    pub fn pceFileCreate(fname: *const c_char, size: c_ulong) -> c_int;
    pub fn pceFileDelete(fname: *const c_char) -> c_int;
    pub fn pceFileApfSave(key: c_int, ptr: *const c_void, len: c_int) -> c_int;
    pub fn pceFileApfLoad(key: c_int, ptr: *mut c_void, len: c_int) -> c_int;
    pub fn pceFileWriteSector(ptr: *mut c_void, len: c_int) -> c_int;

    // ---- Time / RTC ----
    pub fn pceTimeSet(ptime: *const PCETIME);
    pub fn pceTimeGet(ptime: *mut PCETIME);
    pub fn pceTimeSetAlarm(ptime: *const PCEALMTIME) -> c_int;
    pub fn pceTimeGetAlarm(ptime: *mut PCEALMTIME) -> c_int;

    // ---- Power ----
    pub fn pcePowerSetReport(mode: c_int);
    pub fn pcePowerGetStatus(ps: *mut PCEPWRSTAT);
    pub fn pcePowerForceBatt(fn_: c_int);
    pub fn pcePowerEnterStandby(flag: c_int) -> c_int;

    // ---- Infrared (IrDA) ----
    pub fn pceIRStartRx(p_data: *mut c_uchar, len: c_int);
    pub fn pceIRStartTx(p_data: *const c_uchar, len: c_int);
    pub fn pceIRStartRxEx(
        p_data: *mut c_uchar,
        len: c_int,
        mode: c_int,
        callback: Option<unsafe extern "C" fn(rlen: c_int) -> c_int>,
    );
    pub fn pceIRStartTxEx(
        p_data: *const c_uchar,
        len: c_int,
        mode: c_int,
        callback: Option<unsafe extern "C" fn() -> c_int>,
    );
    pub fn pceIRStartRxPulse(
        mode: c_int,
        rxproc: Option<unsafe extern "C" fn(flag: c_int, time: c_ushort)>,
        timeout: c_int,
    );
    pub fn pceIRStartTxPulse(mode: c_int, txproc: Option<unsafe extern "C" fn(flag: c_int) -> c_int>);
    pub fn pceIRStop();
    pub fn pceIRGetStat() -> c_int;

    // ---- USB / USB-COM ----
    pub fn pceUSBDisconnect();
    pub fn pceUSBReconnect();
    pub fn pceUSBSetupMode(mode: c_int, param2: *mut c_void, param3: *mut c_void) -> c_int;
    pub fn pceUSBCOMSetup(puci: *mut USBCOMINFO);
    pub fn pceUSBCOMStartRx(p_data: *mut c_uchar, len: c_int);
    pub fn pceUSBCOMStartTx(p_data: *const c_uchar, len: c_int);
    pub fn pceUSBCOMStop() -> c_int;
    pub fn pceUSBCOMGetStat() -> c_int;

    // ---- System / Debug ----
    pub fn pceSystemGetInfo() -> *const SYSTEMINFO;
    pub fn pceDebugSetMon(mode: c_int);

    // ---- Variadic sprintf (`#define sprintf pcesprintf`) ----
    pub fn pcesprintf(buf: *mut c_char, format: *const c_char, ...) -> c_int;
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

// pceUSBSetupMode
pub const PUM_BASIC: c_int = 0;
pub const PUM_GAMEPAD: c_int = 1;

// pceTimerSetCallback type
pub const PCE_TT_NONE: c_int = 0;
pub const PCE_TT_ONESHOT: c_int = 1;
pub const PCE_TT_PERIODIC: c_int = 2;
/// Context-switcher flag bit: called from a critical section.
pub const PCE_TSCSF_INCRITICAL: c_int = 1;

// pceFontSetType
pub const FC_SPRITE: c_int = -1;

// Wave data types (PCEWAVEINFO.type). ADPCM combos OR the base with flags.
pub const PW_TYPE_8BITPCM: c_uchar = 0;
pub const PW_TYPE_16BITPCM: c_uchar = 1;
pub const PW_TYPE_4BITADPCM: c_uchar = 2;
pub const PW_TYPE_CONT: c_uchar = 0x10; // continuous output
pub const PW_TYPE_ADP_INI: c_uchar = 0x20; // reset ADPCM predictor
pub const PW_TYPE_VR: c_uchar = 0x40; // variable rate
pub const PW_TYPE_ADPCM: c_uchar = PW_TYPE_4BITADPCM | PW_TYPE_ADP_INI;
pub const PW_TYPE_ADPCM_V: c_uchar = PW_TYPE_4BITADPCM | PW_TYPE_VR | PW_TYPE_ADP_INI;
pub const PW_TYPE_ADPCM_NI: c_uchar = PW_TYPE_4BITADPCM;
pub const PW_TYPE_ADPCM_V_NI: c_uchar = PW_TYPE_4BITADPCM | PW_TYPE_VR;

// Wave status (PCEWAVEINFO.stat)
pub const PW_STAT_START: c_uchar = 1;
pub const PW_STAT_END: c_uchar = 2;

// pceAppActiveResponse
pub const AAR_NOACTIVE: c_int = 0;
pub const AAR_ACTIVE: c_int = 1;

// pceFileOpen mode (bit flags)
pub const FOMD_RD: c_int = 1;
pub const FOMD_WR: c_int = 2;

// pceTimeSetAlarm mode (PCEALMTIME.mode)
pub const ALM_STOP: c_ulong = 0;
pub const ALM_EVERYHOUR: c_ulong = 1;
pub const ALM_EVERYDAY: c_ulong = 3;
pub const ALM_ONESHOT: c_ulong = 7;

// pcePowerSetReport
pub const PWR_RPTOFF: c_int = 0;
pub const PWR_RPTON: c_int = 1;

// pceUSBCOMGetStat status bits
pub const UCS_RXWAIT: c_int = 0x001;
pub const UCS_RXING: c_int = 0x002;
pub const UCS_RXDONE: c_int = 0x004;
pub const UCS_TXWAIT: c_int = 0x100;
pub const UCS_TXING: c_int = 0x200;
pub const UCS_TXDONE: c_int = 0x400;
