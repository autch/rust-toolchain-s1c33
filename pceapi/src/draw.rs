//! Graphics primitives (`draw.h`): points, lines, fills, and bitmap/object blits.
//!
//! Coordinates and colors are `long` on the C side; passed here as `i32`. The LCD
//! is [`WIDTH`]×[`HEIGHT`] with 2-bpp grayscale ([`color`]).

use crate::ffi;

pub use crate::ffi::{DRAW_OBJECT, PBMP_FILEHEADER, PIECE_BMP, PIECE_VRAM, RECTP};

/// LCD dimensions from `draw.h` (same as [`crate::lcd::WIDTH`]/`HEIGHT`).
pub const WIDTH: i32 = ffi::DISP_X;
pub const HEIGHT: i32 = ffi::DISP_Y;

/// 2-bpp grayscale colors.
pub mod color {
    use crate::ffi;
    pub const BLACK: i32 = ffi::COLOR_BLACK;
    pub const GRAY_DARK: i32 = ffi::COLOR_GRAY_DARK;
    pub const GRAY_LIGHT: i32 = ffi::COLOR_GRAY_LIGHT;
    pub const WHITE: i32 = ffi::COLOR_WHITE;
    pub const MASK: i32 = ffi::COLOR_MASK;
}

/// Transfer modes for a blit's `param` (low 6 bits); OR with [`REV_X`]/[`REV_Y`].
pub mod mode {
    use crate::ffi;
    pub const NORMAL: i32 = ffi::DRW_NOMAL;
    pub const ADD: i32 = ffi::DRW_ADD;
    pub const SUB: i32 = ffi::DRW_SUB;
    pub const HIGH: i32 = ffi::DRW_HIGH;
    pub const LOW: i32 = ffi::DRW_LOW;
    pub const NOT: i32 = ffi::DRW_NOT;
    pub const OR: i32 = ffi::DRW_OR;
    pub const AND: i32 = ffi::DRW_AND;
    pub const XOR: i32 = ffi::DRW_XOR;
    pub const HALF: i32 = ffi::DRW_HALF;
    pub const LIGHT: i32 = ffi::DRW_LIGHT;
}

/// Horizontal-flip flag for a blit's `param`.
pub const REV_X: i32 = ffi::DRW_REVX;
/// Vertical-flip flag for a blit's `param`.
pub const REV_Y: i32 = ffi::DRW_REVY;

/// Draw a single pixel.
#[inline]
pub fn point(color: i32, x: i32, y: i32) {
    unsafe { ffi::pceLCDPoint(color, x, y) }
}

/// Draw a line from (x1, y1) to (x2, y2).
#[inline]
pub fn line(color: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
    unsafe { ffi::pceLCDLine(color, x1, y1, x2, y2) }
}

/// Fill the rectangle (x1, y1)–(x2, y2).
#[inline]
pub fn paint(color: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
    unsafe { ffi::pceLCDPaint(color, x1, y1, x2, y2) }
}

/// Populate `obj` for a bitmap blit of `src` (see `draw.h` for the parameters).
///
/// # Safety
/// `src` must point at a valid [`PIECE_BMP`] that outlives the eventual
/// [`draw_object`] call.
#[inline]
#[allow(clippy::too_many_arguments)]
pub unsafe fn set_object(
    obj: &mut DRAW_OBJECT,
    src: *mut PIECE_BMP,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    w: i32,
    h: i32,
    param: i32,
) {
    ffi::pceLCDSetObject(obj, src, dx, dy, sx, sy, w, h, param)
}

/// Execute a draw object (passed by value). Returns the kernel result.
///
/// # Safety
/// `dobj`'s `dest` / `src` pointers must be valid for the operation.
#[inline]
pub unsafe fn draw_object(dobj: DRAW_OBJECT) -> i32 {
    ffi::pceLCDDrawObject(dobj)
}
