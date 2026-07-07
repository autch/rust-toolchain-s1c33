//! PFFS file access.
//!
//! Names are passed as `&CStr` (NUL-terminated). Functions return the kernel
//! result code (0 = success) as-is. `FILEINFO` (directory entry) and `FILEACC`
//! (open-file accessor) are re-exported from [`crate::ffi`]; sector-level access
//! (`pceFileOpen`/`ReadSct`/`WriteSct`/`Close`) is available there.

use crate::ffi::{self, FILEINFO};
use core::ffi::CStr;

pub use crate::ffi::{FILEACC, FILEINFO as FileInfo};

/// File open modes (bit flags) for `pceFileOpen`.
pub const READ: i32 = ffi::FOMD_RD;
pub const WRITE: i32 = ffi::FOMD_WR;

/// Load an entire file into `buf`. Returns the kernel result (0 = success).
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
