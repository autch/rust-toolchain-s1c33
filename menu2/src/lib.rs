//! Pure-Rust port of the P/ECE system launcher (`menu2`, the real `startup.pex`).
//!
//! Ported faithfully from `llvm-c33/app/menu2` (`menu.c` + `os.c` + `launch.c`):
//! it scans the PFFS for `.pex` files, draws a 3-column icon grid with a cursor
//! and a caption bubble, and launches the selected app with `pceAppExecFile`.
//! This exercises the `pceapi` graphics + file + pad + app-launch bindings.
//!
//! Mutable state lives in `static mut` globals, as in the C original (P/ECE apps
//! are single-threaded / cooperatively scheduled).

#![no_std]
#![allow(static_mut_refs)]
#![allow(clippy::missing_safety_doc)]

use core::ffi::c_char;
use core::mem::size_of;
use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use pceapi::ffi::{self, pffsFileHEADER, DRAW_OBJECT, FILEACC, FILEINFO, PBMP_FILEHEADER, PIECE_BMP, RECTP};
use pceapi::{app, draw, font, lcd};

const W: usize = 128;
const H: usize = 88;

const FMAX: usize = 90; // max files
const FNMAX: usize = 12 + 1; // filename buffer
const CPMAX: usize = 24 + 1; // caption buffer

// Pad masks (piece.h): held PAD_* and edge-trigger TRG_* (= PAD_* << 8).
const PAD_SELECT: u32 = 0x40;
const TRG_RI: u32 = 0x0100;
const TRG_LF: u32 = 0x0200;
const TRG_DN: u32 = 0x0400;
const TRG_UP: u32 = 0x0800;
const TRG_B: u32 = 0x1000;
const TRG_A: u32 = 0x2000;

// Embedded launcher graphics (extracted from os.c's arrays).
static SYS_BG: [u8; 404] = *include_bytes!("sys_bg.bin");
static DEFAULT_IMG: [u8; 256] = *include_bytes!("default_img.bin");

#[derive(Clone, Copy)]
struct Files {
    name: [u8; FNMAX],
    caption: [u8; CPMAX],
    adrs: u32,
    length: u32,
    iconf: u8,
}
const EMPTY_FILE: Files = Files {
    name: [0; FNMAX],
    caption: [0; CPMAX],
    adrs: 0,
    length: 0,
    iconf: 0,
};

struct OsWork {
    px: i32,
    py: i32,
    frame: i32,
}

const ZERO_BMP: PIECE_BMP = PIECE_BMP {
    header: PBMP_FILEHEADER { head: 0, fsize: 0, bpp: 0, mask: 0, w: 0, h: 0, buf_size: 0 },
    buf: core::ptr::null_mut(),
    mask: core::ptr::null_mut(),
};
const ZERO_OBJ: DRAW_OBJECT = DRAW_OBJECT {
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

static mut VBUFF: [u8; W * H] = [0; W * H];
static mut FILES: [Files; FMAX] = [EMPTY_FILE; FMAX];
static mut FILEC: usize = 0;
static mut POS: usize = 0;
static mut DRAW: i32 = 0;
static mut PBMP: PIECE_BMP = ZERO_BMP;
static mut OS: OsWork = OsWork { px: 0, py: 0, frame: 0 };

// OS_Main sequence.
const OS_INIT: i32 = 0;
const OS_MAIN: i32 = 2;
static mut SEQUENCE: i32 = OS_INIT;

// Speech-bubble line endpoints (from os.c).
const PT1: [[i32; 4]; 3] = [[22, 24, 26, 28], [18, 16, 14, 12], [17, 12, 7, 2]];
const PT2: [[i32; 4]; 3] = [[23, 28, 33, 38], [19, 20, 21, 22], [18, 16, 14, 12]];
const OFT: [i32; 3] = [0, 45, 85];

// ---- launch.c ----

/// Read a file's PFFS header (maps it via a NUL-buffer read; the returned pointer
/// is the kernel-mapped file image). Returns null on failure or a too-short file.
unsafe fn getfh(fname: *const c_char) -> *const pffsFileHEADER {
    let mut facc = FILEACC::default();
    if ffi::pceFileOpen(&mut facc, fname, ffi::FOMD_RD) != 0 {
        return core::ptr::null();
    }
    let s = ffi::pceFileReadSct(&mut facc, core::ptr::null_mut(), 0, 4096);
    ffi::pceFileClose(&mut facc);
    if s < 24 {
        return core::ptr::null();
    }
    facc.aptr as *const pffsFileHEADER
}

/// C string length (bytes up to NUL).
unsafe fn cstrlen(p: *const u8) -> usize {
    let mut n = 0;
    while *p.add(n) != 0 {
        n += 1;
    }
    n
}

/// Is `name` a launchable ".pex" other than the launcher itself?
unsafe fn nameck(name: *const u8) -> bool {
    let len = cstrlen(name);
    if len < 4 {
        return false;
    }
    let ext = core::slice::from_raw_parts(name.add(len - 4), 4);
    if ext != b".pex" {
        return false;
    }
    // exclude "startup.pex"
    let full = core::slice::from_raw_parts(name, len);
    full != b"startup.pex"
}

/// Copy a NUL-terminated string into `dst` (like strncpy, capped at dst.len()).
unsafe fn strncpy_into(dst: &mut [u8], src: *const u8) {
    let n = cstrlen(src);
    let m = if n < dst.len() { n } else { dst.len() - 1 };
    for i in 0..m {
        dst[i] = *src.add(i);
    }
    dst[m] = 0;
}

unsafe fn getdir() {
    let mut fi = FILEINFO::default();
    let mut i = 0usize;
    font::set_pos(0, 10);
    ffi::pceFileFindOpen(&mut fi);
    while ffi::pceFileFindNext(&mut fi) != 0 {
        let name = fi.filename.as_ptr() as *const u8;
        if nameck(name) {
            let pfh = getfh(fi.filename.as_ptr());
            if !pfh.is_null() {
                let pf = &mut FILES[i];
                strncpy_into(&mut pf.name, name);
                let cap = (pfh as *const u8).add((*pfh).ofs_name as usize);
                strncpy_into(&mut pf.caption, cap);
                pf.iconf = ((*pfh).ofs_icon != 0) as u8;
                pf.length = fi.length;
                pf.adrs = (*pfh).top_adrs;
                i += 1;
                if i >= FMAX {
                    break;
                }
            }
        }
    }
    ffi::pceFileFindClose(&mut fi);
    FILEC = i;
}

/// Copy a file's 32x32 2bpp icon (256 bytes) into `buff`. Returns 0 on success.
unsafe fn geticondata(fname: *const c_char, buff: *mut u8) -> i32 {
    let pfh = getfh(fname);
    if !pfh.is_null() && (*pfh).ofs_icon != 0 {
        let icon = (pfh as *const u8).add((*pfh).ofs_icon as usize);
        core::ptr::copy_nonoverlapping(icon, buff, (32 / 4) * 32);
        return 0;
    }
    -1
}

unsafe fn run(pf: &Files) {
    app::exec_file(core::ffi::CStr::from_ptr(pf.name.as_ptr() as *const c_char), 0);
}

unsafe fn go_standby() {
    pceapi::power::enter_standby(0);
    pceapi::usb::reconnect();
}

// ---- os.c ----

/// Unpack a 2bpp (1 byte = 4 pixels) image directly into VRAM (1 byte/pixel).
unsafe fn bitmapdraw(x: i32, y: i32, w: i32, h: i32, mut adr: *const u8) {
    let vram = lcd::current_buffer();
    let mut j = y;
    let ymax = if y + h < 88 { y + h } else { 88 };
    while j < ymax {
        for i in 0..(w / 4) {
            let ptr = vram.offset((x + i * 4) as isize + 128 * j as isize);
            let b = *adr;
            *ptr.add(0) = (b >> 6) & 0x3;
            *ptr.add(1) = (b >> 4) & 0x3;
            *ptr.add(2) = (b >> 2) & 0x3;
            *ptr.add(3) = b & 0x3;
            adr = adr.add(1);
        }
        j += 1;
    }
}

/// Point PBMP at the embedded sys_bg image (parses its PBMP header).
unsafe fn set_sysbg() {
    let src = SYS_BG.as_ptr();
    PBMP.header = core::ptr::read_unaligned(src as *const PBMP_FILEHEADER);
    PBMP.buf = src.add(size_of::<PBMP_FILEHEADER>()) as *mut u8;
    let px = (PBMP.header.h as usize) * (PBMP.header.w as usize) / 4;
    PBMP.mask = src.add(size_of::<PBMP_FILEHEADER>() + px) as *mut u8;
}

unsafe fn blit(dx: i32, dy: i32, sx: i32, sy: i32, w: i32, h: i32) {
    let mut obj = ZERO_OBJ;
    draw::set_object(&mut obj, addr_of_mut!(PBMP), dx, dy, sx, sy, w, h, draw::mode::NORMAL);
    draw::draw_object(obj);
}

unsafe fn draw_frame() {
    let vram = lcd::current_buffer();

    // background stripes
    let vraml = vram as *mut u32;
    let mut j = 0isize;
    while j < 88 {
        for i in 0..32isize {
            *vraml.offset(i + j * 32) = 0x0001_0001;
            *vraml.offset(i + 32 + j * 32) = 0x0100_0100;
        }
        j += 2;
    }

    set_sysbg();

    if OS.frame == 0 {
        core::ptr::write_bytes(vram, 2, 128 * 8);
        blit(0, 0, 64, 0, 20, 8);
        blit(102, 0, 84, 0, 24, 8);
    } else {
        blit(8, 0, 0, 0, 16, 8);
        blit(104, 0, 0, 0, 16, 8);
    }
    if FILEC as i32 <= OS.frame * 3 + 6 || (OS.py + 1) * 3 >= FILEC as i32 {
        core::ptr::write_bytes(vram.add(128 * 80), 2, 128 * 8);
        blit(2, 80, 84, 0, 24, 8);
        blit(106, 80, 108, 0, 20, 8);
    } else {
        blit(8, 80, 16, 0, 16, 8);
        blit(104, 80, 16, 0, 16, 8);
    }

    // icons
    let mut buff = [0u8; 256];
    let mut i = OS.frame * 3;
    while i < FILEC as i32 {
        let idx = i as usize;
        let icon: *const u8 = if FILES[idx].iconf != 0 {
            geticondata(FILES[idx].name.as_ptr() as *const c_char, buff.as_mut_ptr());
            buff.as_ptr()
        } else {
            DEFAULT_IMG.as_ptr()
        };
        bitmapdraw(4 + 44 * ((i - OS.frame * 3) % 3), 8 + 36 * ((i - OS.frame * 3) / 3), 32, 32, icon);
        i += 1;
    }

    // selection cursor
    let off = OS.py - OS.frame;
    set_sysbg();
    blit(4 + 44 * OS.px, 8 + 36 * off, 32, 0, 8, 8);
    blit(28 + 44 * OS.px, 8 + 36 * off, 40, 0, 8, 8);
    blit(4 + 44 * OS.px, 32 + 36 * off, 48, 0, 8, 8);
    blit(28 + 44 * OS.px, 32 + 36 * off, 56, 0, 8, 8);

    // caption bubble
    let px = OS.px as usize;
    if off != 0 {
        let y = 44;
        for x in 0..4usize {
            draw::line(2, OFT[px] + PT1[px][x], y - 1 - x as i32, OFT[px] + PT2[px][x], y - 1 - x as i32);
        }
        draw::paint(2, 21, 20, 86, 20);
    } else {
        let y = 39;
        for x in 0..4usize {
            draw::line(2, OFT[px] + PT1[px][x], y + 1 + x as i32, OFT[px] + PT2[px][x], y + 1 + x as i32);
        }
        draw::paint(2, 21, 44, 86, 20);
    }

    // selected file text
    font::set_bk_color(2);
    font::set_tx_color(0);
    let cap = FILES[POS].caption.as_ptr();
    let nam = FILES[POS].name.as_ptr();
    let cap_y = if off != 0 { 20 } else { 44 };
    let nam_y = if off != 0 { 30 } else { 54 };
    font::set_pos(64 - (cstrlen(cap) as i32) * 5 / 2, cap_y);
    printf_s(cap);
    font::set_pos(64 - (cstrlen(nam) as i32) * 5 / 2, nam_y);
    printf_s(nam);
    font::set_bk_color(0);
    font::set_tx_color(3);
}

/// `pceFontPrintf("%s\n", s)`.
unsafe fn printf_s(s: *const u8) {
    ffi::pceFontPrintf(b"%s\n\0".as_ptr() as *const c_char, s as *const c_char);
}

unsafe fn renewpos(dir: u32) {
    let num = FILEC as i32;
    // clamp out-of-range position
    if 3 * OS.py + OS.px > num {
        OS.px = num % 3;
        OS.py = num / 3;
        OS.frame = if OS.py <= 0 { 0 } else { OS.py - 1 };
    }
    if dir & TRG_LF != 0 {
        if (num + 2) / 3 - 1 > OS.py || num % 3 == 0 {
            OS.px = if OS.px == 0 { 2 } else { OS.px - 1 };
        } else if num % 3 == 1 {
            OS.px = 0;
        } else {
            OS.px = if OS.px == 0 { 1 } else { 0 };
        }
        POS = (OS.px + OS.py * 3) as usize;
    }
    if dir & TRG_RI != 0 {
        if (num + 2) / 3 - 1 > OS.py || num % 3 == 0 {
            OS.px = if OS.px == 2 { 0 } else { OS.px + 1 };
        } else if num % 3 == 1 {
            OS.px = 0;
        } else {
            OS.px = if OS.px == 0 { 1 } else { 0 };
        }
        POS = (OS.px + OS.py * 3) as usize;
    }
    if dir & TRG_UP != 0 {
        if OS.py > 0 {
            OS.py -= 1;
            OS.frame = if OS.py < 1 { 0 } else { OS.py };
        }
        POS = (OS.px + OS.py * 3) as usize;
    }
    if dir & TRG_DN != 0 {
        if (OS.py + 1) * 3 + OS.px < num {
            OS.py += 1;
            if OS.py > 1 {
                OS.frame = OS.py - 1;
            }
        } else if (OS.py + 1) * 3 < num {
            OS.py += 1;
            if OS.py > 1 {
                OS.frame = OS.py - 1;
            }
            OS.px = (num + 2) % 3;
        }
        POS = (OS.px + OS.py * 3) as usize;
    }
    if dir & TRG_A != 0 {
        run(&FILES[POS]);
    }
}

unsafe fn os_main() {
    let pad = ffi::pcePadGet() as u32;
    let vram = lcd::current_buffer();
    match SEQUENCE {
        OS_MAIN => {
            if pad != 0 || DRAW != 0 {
                renewpos(pad);
                draw_frame();
                DRAW = 0;
            }
        }
        _ => {
            // OS_INIT
            core::ptr::write_bytes(vram, 0, 128 * 88);
            OS.px = 0;
            OS.py = 0;
            OS.frame = 0;
            renewpos(0);
            getdir();
            SEQUENCE = OS_MAIN;
            POS = 0;
            draw_frame();
        }
    }
}

// ---- menu.c ----

#[no_mangle]
pub extern "C" fn pceAppInit() {
    unsafe {
        lcd::disp_stop();
        lcd::set_buffer(addr_of_mut!(VBUFF) as *mut u8);
        app::set_proc_period(40);
        lcd::trans();
        lcd::disp_start();
        pceapi::cpu::set_speed(pceapi::cpu::Speed::Half);
    }
}

#[no_mangle]
pub extern "C" fn pceAppProc(_cnt: i32) {
    unsafe {
        let a = ffi::pcePadGet() as u32;
        if a != 0 {
            getdir();
        }
        if (a & TRG_B) != 0 && (a & PAD_SELECT) != 0 {
            go_standby();
            DRAW = 4;
        }
        os_main();
        lcd::trans();
    }
}

#[no_mangle]
pub extern "C" fn pceAppExit() {
    pceapi::cpu::set_speed(pceapi::cpu::Speed::Normal);
}

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
