//! Minimal P/ECE application in pure Rust.
//!
//! This is the end-to-end ABI proof for the `s1c33-none-piece` target: it
//! provides the four callbacks the P/ECE crt0 expects (`pceAppInit`,
//! `pceAppProc`, `pceAppExit`, `pceAppNotify`), draws a string to the LCD, and
//! links against the language-agnostic sysroot artifacts (crt0.o, libpceapi.a).
//!
//! Modelled on the SDK's `hello.c`. crt0 clears BSS before `pceAppInit` runs,
//! so the zero-initialised `VBUFF` needs no runtime memset.

#![no_std]

use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

// ---- P/ECE kernel API (S5U1C33000C ABI), from sdk/include/piece.h ----
extern "C" {
    fn pceLCDDispStop();
    fn pceLCDDispStart();
    fn pceLCDSetBuffer(pbuff: *mut u8) -> *mut u8;
    fn pceLCDTrans();
    fn pceAppSetProcPeriod(period: i32) -> i32;
    fn pceFontSetPos(x: i32, y: i32);
    fn pceFontPutStr(pstr: *const u8) -> i32;
}

const LCD_W: usize = 128;
const LCD_H: usize = 88;

/// Off-screen framebuffer handed to the kernel. Lives in .bss (crt0 zeroes it).
///
/// A bare `[u8; N]` has alignment 1, but the `s1c33-none-piece` target sets
/// `min_global_align = 32 bits`, so rustc emits every global at ≥4-byte
/// alignment (matching gcc33 / clang's `MinGlobalAlign=32`). This matters
/// because the kernel's `pceLCDTrans` reads the framebuffer with 32-bit loads,
/// which the S1C33000 traps on a misaligned base.
static mut VBUFF: [u8; LCD_W * LCD_H] = [0; LCD_W * LCD_H];

#[no_mangle]
pub extern "C" fn pceAppInit() {
    unsafe {
        pceLCDDispStop();
        pceLCDSetBuffer(addr_of_mut!(VBUFF) as *mut u8);
        pceAppSetProcPeriod(80);
        pceFontSetPos(0, 0);
        pceFontPutStr(b"Hello from Rust!\0".as_ptr());
        pceLCDDispStart();
    }
}

#[no_mangle]
pub extern "C" fn pceAppProc(_cnt: i32) {
    // Push the framebuffer to the LCD each tick.
    unsafe {
        pceLCDTrans();
    }
}

#[no_mangle]
pub extern "C" fn pceAppExit() {}

#[no_mangle]
pub extern "C" fn pceAppNotify(_type: i32, _param: i32) -> i32 {
    0
}

/// P/ECE apps never exit conventionally — the kernel reclaims the app by reset,
/// so `_exit` has nowhere to return to and simply spins (mirrors newlib's
/// `sys/s1c33/_exit.c`). picolibc's `abort()` — pulled in by compiler_builtins'
/// cold paths — calls `_exit`, so this stub is required to link.
#[no_mangle]
pub extern "C" fn _exit(_status: i32) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
