//! LCD framebuffer control.

use crate::ffi;

/// LCD dimensions (pixels). The framebuffer is `WIDTH * HEIGHT` bytes.
pub const WIDTH: usize = 128;
pub const HEIGHT: usize = 88;

/// Start driving the LCD from the current framebuffer.
#[inline]
pub fn disp_start() {
    unsafe { ffi::pceLCDDispStart() }
}

/// Stop driving the LCD.
#[inline]
pub fn disp_stop() {
    unsafe { ffi::pceLCDDispStop() }
}

/// Transfer the framebuffer set by [`set_buffer`] to the LCD.
#[inline]
pub fn trans() {
    unsafe { ffi::pceLCDTrans() }
}

/// Tell the kernel which framebuffer to display.
///
/// # Safety
/// The kernel retains `buf` across calls, so it must point at `WIDTH * HEIGHT`
/// bytes that stay valid and **4-byte aligned** for as long as the display is
/// active (typically a `static`). The S1C33000 traps misaligned word access,
/// which `pceLCDTrans` performs on this buffer.
#[inline]
pub unsafe fn set_buffer(buf: *mut u8) {
    ffi::pceLCDSetBuffer(buf);
}

/// Set LCD brightness; returns the previous value.
#[inline]
pub fn set_bright(bright: i32) -> i32 {
    unsafe { ffi::pceLCDSetBright(bright) }
}

/// Set LCD orientation; returns the previous value.
#[inline]
pub fn set_orientation(dir: i32) -> i32 {
    unsafe { ffi::pceLCDSetOrientation(dir) }
}
