//! Minimal P/ECE application in pure Rust.
//!
//! End-to-end ABI proof for the `s1c33-none-piece` target: it provides the four
//! callbacks the P/ECE crt0 expects (`pceAppInit`, `pceAppProc`, `pceAppExit`,
//! `pceAppNotify`), sets up a `#[global_allocator]` on the kernel heap, exercises
//! `alloc` (Vec + String), and draws to the LCD. Links against the
//! language-agnostic sysroot artifacts (crt0.o, libpceapi.a).
//!
//! Modelled on the SDK's `hello.c`. crt0 clears BSS before `pceAppInit` runs and
//! the kernel calls `ResetHeap` before that, so both `VBUFF` and the heap are
//! ready by the time `pceAppInit` runs.

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

mod heap;

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

/// Draw a Rust `&str` at (x, y). `pceFontPutStr` wants a NUL-terminated C string,
/// so build a heap copy with a trailing NUL (this also exercises the allocator).
fn draw(x: i32, y: i32, s: &str) {
    let mut buf: Vec<u8> = Vec::with_capacity(s.len() + 1);
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
    unsafe {
        pceFontSetPos(x, y);
        pceFontPutStr(buf.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn pceAppInit() {
    unsafe {
        pceLCDDispStop();
        pceLCDSetBuffer(addr_of_mut!(VBUFF) as *mut u8);
        pceAppSetProcPeriod(80);
    }

    // Exercise the kernel-heap allocator: build a Vec, reduce it, and format the
    // result into a heap String. All of this allocs/frees via pceHeapAlloc/Free.
    //
    // `black_box` the range end so the optimizer cannot const-fold the sum to a
    // literal (which it does for `(1..=5).sum()`): the length is then unknown at
    // compile time, forcing `collect` to fill the heap buffer and `sum` to read it
    // back at runtime. So the displayed number is genuinely computed from heap data.
    let n = core::hint::black_box(5u32);
    let v: Vec<u32> = (1..=n).collect();
    let sum: u32 = v.iter().copied().sum();
    let line = format!("heap sum = {}", sum);

    draw(0, 0, "Hello from Rust!");
    draw(0, 12, &line);

    unsafe {
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
