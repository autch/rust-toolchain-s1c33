//! `jien` — a port of the P/ECE sample app (a bouncing sprite) to pure Rust.
//!
//! Exercises the `pceapi` graphics bindings (`draw::set_object`/`draw_object`
//! with `DRAW_OBJECT`/`PIECE_BMP`) end-to-end. The bitmap is embedded from the
//! original `jien_bmp.pgd`; since the SDK's `PBM_*` loaders are not kernel
//! services, the app parses the `PBMP` header by hand (as the original C did).
//!
//! Faithful to the C original, this keeps its mutable state in `static mut`
//! globals (P/ECE apps are single-threaded and cooperatively scheduled).

#![no_std]
#![allow(static_mut_refs)]

use core::ffi::{c_char, c_int};
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use pceapi::ffi::{DRAW_OBJECT, PBMP_FILEHEADER, PIECE_BMP, RECTP};
use pceapi::{app, draw, font, lcd};

const W: usize = 128;
const H: usize = 88;

/// Off-screen framebuffer (4-byte aligned via the target's min_global_align).
static mut VSCREEN: [u8; W * H] = [0; W * H];

/// The sprite bitmap, embedded from the original binary (`PBMP` format, 40×16).
static JIEN_BMP: [u8; 260] = *include_bytes!("jien_bmp.pgd");

static mut OBJ: DRAW_OBJECT = DRAW_OBJECT {
    dest: core::ptr::null_mut(),
    dx: 0,
    dy: 0,
    dw: 0,
    dh: 0,
    src: core::ptr::null_mut(),
    sx: 0,
    sy: 0,
    clip: RECTP { left: 0, top: 0, right: 0, bottom: 0 },
    disp: 0,
    param: 0,
    type_: 0,
    layer: 0,
};
static mut JIEN: PIECE_BMP = PIECE_BMP {
    header: PBMP_FILEHEADER { head: 0, fsize: 0, bpp: 0, mask: 0, w: 0, h: 0, buf_size: 0 },
    buf: core::ptr::null_mut(),
    mask: core::ptr::null_mut(),
};

static mut X: i32 = 0;
static mut Y: i32 = 0;
static mut DX: i32 = 0;
static mut DY: i32 = 0;
static mut DIRTY: bool = false;

/// Parse a `PBMP` bitmap out of a static buffer, pointing `buf`/`mask` into it.
/// (Replaces the SDK's `PBM_*` loaders, which are not kernel services.)
unsafe fn load_bitmap() {
    let src = JIEN_BMP.as_ptr();
    JIEN.header = core::ptr::read_unaligned(src as *const PBMP_FILEHEADER);
    let p = src.add(size_of::<PBMP_FILEHEADER>());
    JIEN.buf = p as *mut u8;
    let pixels = (JIEN.header.w as usize) * (JIEN.header.h as usize);
    JIEN.mask = p.add(pixels >> 2) as *mut u8; // 2 bits/pixel
}

unsafe fn move_sprite() {
    let w = JIEN.header.w as i32;
    let h = JIEN.header.h as i32;
    if X + DX < 0 || X + DX + w > W as i32 {
        DX = -DX;
    }
    if Y + DY < 0 || Y + DY + h > H as i32 {
        DY = -DY;
    }
    X += DX;
    Y += DY;
}

unsafe fn draw_sprite() {
    let w = JIEN.header.w as i32;
    let h = JIEN.header.h as i32;
    draw::set_object(&mut OBJ, &mut JIEN, X, Y, 0, 0, w, h, draw::mode::NORMAL);
    draw::draw_object(OBJ); // DRAW_OBJECT is Copy — passed by value
    DIRTY = true;
    move_sprite();
}

unsafe fn refresh() {
    if !DIRTY {
        return;
    }
    lcd::trans();
    DIRTY = false;
}

unsafe fn cls() {
    core::ptr::write_bytes(addr_of_mut!(VSCREEN) as *mut u8, 0, W * H);
    DIRTY = true;
}

#[no_mangle]
pub extern "C" fn pceAppInit() {
    unsafe {
        DIRTY = true;
        lcd::disp_stop();
        lcd::set_buffer(addr_of_mut!(VSCREEN) as *mut u8);
        app::set_proc_period(50);
        cls();

        load_bitmap();
        font::set_pos(0, 0);
        pceapi::ffi::pceFontPrintf(
            b"w:%d,h:%d\0".as_ptr() as *const c_char,
            JIEN.header.w as c_int,
            JIEN.header.h as c_int,
        );

        X = 0;
        Y = 0;
        DX = 1;
        DY = 1;
        draw_sprite();

        refresh();
        lcd::disp_start();
    }
}

#[no_mangle]
pub extern "C" fn pceAppProc(_cnt: i32) {
    unsafe {
        draw_sprite();
        refresh();
    }
}

#[no_mangle]
pub extern "C" fn pceAppExit() {}

#[no_mangle]
pub extern "C" fn pceAppNotify(_type: i32, _param: i32) -> i32 {
    app::response::IGNORE
}

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
