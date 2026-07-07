//! PFFS file access.
//!
//! Names are passed as `&CStr` (NUL-terminated). Functions return the kernel
//! result code (0 = success) as-is.
//!
//! For ordinary reading/writing use the **sector API** ([`open`] → [`read_sct`] /
//! [`write_sct`] → [`close`]), which transfers data in sector units. [`load`] is a
//! special loader, not a plain read — see its note.

use crate::ffi::{self, FILEINFO};
use core::ffi::CStr;

pub use crate::ffi::{FILEACC, FILEINFO as FileInfo};

/// File open modes (bit flags) for [`open`].
pub const READ: i32 = ffi::FOMD_RD;
pub const WRITE: i32 = ffi::FOMD_WR;

/// **Special loader — not a general file read.** `pceFileLoad` copies a file to
/// `buf`, but for an executable (`.pex`) it *decompresses* it to that address.
/// It does **not** run anything (launch apps with `pceAppExecFile`), and its
/// decompress-on-load behaviour makes it easy to misuse — prefer the sector API
/// ([`open`]/[`read_sct`]/[`write_sct`]/[`close`]) for ordinary file I/O. Returns
/// the kernel result (0 = success).
#[inline]
pub fn load(fname: &CStr, buf: &mut [u8]) -> i32 {
    unsafe { ffi::pceFileLoad(fname.as_ptr(), buf.as_mut_ptr().cast()) }
}

/// Create a file of `size` bytes. Returns the kernel result.
#[inline]
pub fn create(fname: &CStr, size: u32) -> i32 {
    unsafe { ffi::pceFileCreate(fname.as_ptr(), size) }
}

/// Delete a file. Returns the kernel result.
#[inline]
pub fn delete(fname: &CStr) -> i32 {
    unsafe { ffi::pceFileDelete(fname.as_ptr()) }
}

// ---- sector I/O (the normal read/write path) ----

/// Open `fname` for sector I/O. `mode` is [`READ`] and/or [`WRITE`]. Start `fa`
/// from `FILEACC::default()`; the kernel fills it in. Returns the kernel result.
#[inline]
pub fn open(fa: &mut FILEACC, fname: &CStr, mode: i32) -> i32 {
    unsafe { ffi::pceFileOpen(fa, fname.as_ptr(), mode) }
}

/// Read `len` sectors starting at sector `sct` into `buf` (which must be large
/// enough for `len` sectors). Returns the kernel result.
#[inline]
pub fn read_sct(fa: &mut FILEACC, buf: &mut [u8], sct: i32, len: i32) -> i32 {
    unsafe { ffi::pceFileReadSct(fa, buf.as_mut_ptr().cast(), sct, len) }
}

/// Write `len` sectors from `buf` starting at sector `sct`. Returns the kernel result.
#[inline]
pub fn write_sct(fa: &mut FILEACC, buf: &[u8], sct: i32, len: i32) -> i32 {
    unsafe { ffi::pceFileWriteSct(fa, buf.as_ptr().cast(), sct, len) }
}

/// Close a file opened with [`open`]. Returns the kernel result.
#[inline]
pub fn close(fa: &mut FILEACC) -> i32 {
    unsafe { ffi::pceFileClose(fa) }
}

// ---- application preferences (save data) ----

/// Save a small key-indexed blob (application preferences / save data). Returns
/// the kernel result.
#[inline]
pub fn apf_save(key: i32, data: &[u8]) -> i32 {
    unsafe { ffi::pceFileApfSave(key, data.as_ptr().cast(), data.len() as i32) }
}

/// Load a key-indexed blob into `buf`. Returns the kernel result.
#[inline]
pub fn apf_load(key: i32, buf: &mut [u8]) -> i32 {
    unsafe { ffi::pceFileApfLoad(key, buf.as_mut_ptr().cast(), buf.len() as i32) }
}

// ---- directory scan ----

/// Begin a directory scan; fills `fi` with the first entry. 0 = an entry was
/// found. Follow with [`find_next`] and finish with [`find_close`].
#[inline]
pub fn find_open(fi: &mut FILEINFO) -> i32 {
    unsafe { ffi::pceFileFindOpen(fi) }
}

/// Fetch the next directory entry into `fi`. 0 = an entry was found.
#[inline]
pub fn find_next(fi: &mut FILEINFO) -> i32 {
    unsafe { ffi::pceFileFindNext(fi) }
}

/// End a directory scan started with [`find_open`].
#[inline]
pub fn find_close(fi: &mut FILEINFO) -> i32 {
    unsafe { ffi::pceFileFindClose(fi) }
}
