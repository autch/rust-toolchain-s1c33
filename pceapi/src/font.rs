//! Built-in font drawing.

use crate::ffi;
use core::ffi::CStr;

/// Set the text cursor position (pixels).
#[inline]
pub fn set_pos(x: i32, y: i32) {
    unsafe { ffi::pceFontSetPos(x, y) }
}

/// Set the text (foreground) color.
#[inline]
pub fn set_tx_color(color: i32) {
    unsafe { ffi::pceFontSetTxColor(color) }
}

/// Set the background color.
#[inline]
pub fn set_bk_color(color: i32) {
    unsafe { ffi::pceFontSetBkColor(color) }
}

/// Set the font type.
#[inline]
pub fn set_type(ty: i32) {
    unsafe { ffi::pceFontSetType(ty) }
}

/// Draw a NUL-terminated C string at the current position (no allocation).
#[inline]
pub fn put_cstr(s: &CStr) -> i32 {
    unsafe { ffi::pceFontPutStr(s.as_ptr()) }
}

/// Draw a Rust string, allocating a NUL-terminated temporary on the heap.
///
/// Requires a global allocator (see [`crate::heap`]).
#[cfg(feature = "alloc")]
pub fn put_str(s: &str) -> i32 {
    let mut buf = alloc::vec::Vec::with_capacity(s.len() + 1);
    buf.extend_from_slice(s.as_bytes());
    buf.push(0);
    unsafe { ffi::pceFontPutStr(buf.as_ptr() as *const core::ffi::c_char) }
}
