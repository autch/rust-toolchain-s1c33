//! Minimal P/ECE application in pure Rust.
//!
//! End-to-end ABI proof for the `s1c33-none-piece` target: it provides the four
//! callbacks the P/ECE crt0 expects (`pceAppInit`, `pceAppProc`, `pceAppExit`,
//! `pceAppNotify`), installs the kernel-heap allocator, exercises `alloc`
//! (Vec + String), and draws to the LCD via the [`pceapi`] bindings.
//!
//! crt0 clears BSS before `pceAppInit` runs and the kernel calls `ResetHeap`
//! before that, so both `VBUFF` and the heap are ready by then.

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::vec::Vec;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use pceapi::{app, font, lcd};

/// Install the kernel heap as the global allocator (see `pceapi::heap`).
#[global_allocator]
static ALLOCATOR: pceapi::heap::PceHeap = pceapi::heap::PceHeap;

/// Off-screen framebuffer handed to the kernel. Lives in .bss (crt0 zeroes it).
/// The `s1c33-none-piece` target's `min_global_align = 32 bits` keeps it 4-byte
/// aligned, as `pceLCDTrans`'s 32-bit reads require.
static mut VBUFF: [u8; lcd::WIDTH * lcd::HEIGHT] = [0; lcd::WIDTH * lcd::HEIGHT];

/// Draw a string at (x, y) (allocates a NUL-terminated temporary).
fn draw(x: i32, y: i32, s: &str) {
    font::set_pos(x, y);
    font::put_str(s);
}

#[no_mangle]
pub extern "C" fn pceAppInit() {
    lcd::disp_stop();
    // SAFETY: VBUFF is a 'static, 4-byte-aligned framebuffer of the right size.
    unsafe { lcd::set_buffer(addr_of_mut!(VBUFF) as *mut u8) };
    app::set_proc_period(80);

    // Exercise the kernel-heap allocator: build a Vec, reduce it, format the
    // result into a heap String. `black_box` the range end so the sum is a real
    // runtime read of the heap buffer, not the const-folded literal 15.
    let n = core::hint::black_box(5u32);
    let v: Vec<u32> = (1..=n).collect();
    let sum: u32 = v.iter().copied().sum();
    let line = format!("heap sum = {}", sum);

    draw(0, 0, "Hello from Rust!");
    draw(0, 12, &line);

    // c-variadic ABI check: call the kernel's printf with several int varargs.
    // On s1c33 the format is the last named arg, so it *and* every vararg go on
    // the stack — this exercises the variadic calling convention. If a slot were
    // off by one the rendered numbers would be garbage; expect "va: 11 22 33 44".
    font::set_pos(0, 24);
    unsafe {
        pceapi::ffi::pceFontPrintf(
            b"va: %d %d %d %d\0".as_ptr() as *const core::ffi::c_char,
            11i32,
            22i32,
            33i32,
            44i32,
        );
    }

    lcd::disp_start();
}

#[no_mangle]
pub extern "C" fn pceAppProc(_cnt: i32) {
    // Push the framebuffer to the LCD each tick.
    lcd::trans();
}

#[no_mangle]
pub extern "C" fn pceAppExit() {}

#[no_mangle]
pub extern "C" fn pceAppNotify(_type: i32, _param: i32) -> i32 {
    app::response::IGNORE
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
