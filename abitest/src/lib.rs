//! Bare-metal ABI unit tests for the `s1c33-none-piece` target.
//!
//! Runs under `piece-emu` without the P/ECE kernel: the emulator sets PC to the
//! ELF entry (`_start` from the bare-metal `crt0.s`/`crt_init.c`), which calls
//! `main()` and writes its return value to the semihosting `TEST_RESULT` port
//! (0 = PASS, non-zero = FAIL). The emulator turns that into its process exit
//! code, so `piece-emu --max-cycles N abitest.elf` is a pass/fail check.
//!
//! Each `check!` that fails returns a distinct code (also printed by the
//! emulator), so a failure says *which* assertion broke.

#![no_std]

use core::panic::PanicInfo;

/// Semihosting port base (see `piece-emu/src/tests/bare_metal/semihosting.h`).
const SEMI_BASE: usize = 0x0006_0000;
const SEMI_TEST_RESULT: *mut u32 = (SEMI_BASE + 0x08) as *mut u32;

/// Write `code` to TEST_RESULT and halt (the emulator stops the sim on write).
fn semi_exit(code: u32) -> ! {
    unsafe { core::ptr::write_volatile(SEMI_TEST_RESULT, code) };
    loop {
        core::hint::spin_loop();
    }
}

/// Return the given failure `code` from `main` if `cond` is false.
macro_rules! check {
    ($cond:expr, $code:expr) => {
        if !$cond {
            return $code;
        }
    };
}

// ---- C-side ABI fixtures (abitest/csrc/abi_test.c, built by clang) ----
#[repr(C)]
struct P {
    x: i32,
    y: i32,
}
#[repr(C)]
struct One {
    v: i32,
}
#[repr(C)]
struct B {
    b: u8,
}
extern "C" {
    fn sum_p(p: P) -> i32;
    fn one_val(o: One) -> i32;
    fn b_val(s: B) -> i32;
    fn sum_va(count: i32, ...) -> i32;
}

/// Entry called by `crt_init.c`'s `_start_c`; return 0 for PASS.
#[no_mangle]
pub extern "C" fn main() -> i32 {
    // struct-by-value: multi-field struct passed on the stack.
    check!(unsafe { sum_p(P { x: 3, y: 4 }) } == 7, 1);
    // §3.5: single 32-bit element passed in a register.
    check!(unsafe { one_val(One { v: 42 }) } == 42, 2);
    // §3.5: single 8-bit element coerced + high-bit-packed.
    check!(unsafe { b_val(B { b: 200 }) } == 200, 3);
    // c-variadic (caller side): named `count` in a register, varargs on the stack.
    check!(unsafe { sum_va(4, 11, 22, 33, 44) } == 110, 4);
    // more varargs than argument registers, to exercise stack spill ordering.
    check!(unsafe { sum_va(6, 1, 2, 3, 4, 5, 6) } == 21, 5);
    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // A panic in a test is a failure with a sentinel code.
    semi_exit(0xFF)
}
