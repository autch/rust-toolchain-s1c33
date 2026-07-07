//! Global allocator backed by the P/ECE kernel heap (`pceHeapAlloc` /
//! `pceHeapFree`).
//!
//! Install it in your app:
//! ```ignore
//! #[global_allocator]
//! static ALLOCATOR: pceapi::heap::PceHeap = pceapi::heap::PceHeap;
//! ```
//!
//! Why the kernel heap rather than libc `malloc`: `pceHeapAlloc` bounds-checks
//! against `sram_end` and returns NULL on exhaustion (→ Rust's alloc-error path),
//! whereas libc's `sbrk`-backed malloc has no upper bound and silently corrupts
//! kernel state on overrun. C/C++ were forced onto libc malloc because a correct
//! C `realloc()` must relocate and `pceHeapRealloc` cannot; Rust's `GlobalAlloc`
//! only needs `alloc`/`dealloc` and composes `realloc` itself, so it is free to
//! use the safer kernel heap. As long as the app never calls libc `malloc`, the
//! kernel heap owns the whole `[bss_end, sram_end)` region.
//!
//! `pceHeapAlloc` guarantees only 4-byte alignment; larger alignments are handled
//! by over-allocating and stashing the real base pointer just below the returned
//! (aligned) pointer, to be recovered on free.

use crate::ffi;
use core::alloc::{GlobalAlloc, Layout};
use core::ffi::c_void;
use core::mem::size_of;
use core::ptr;

/// Alignment `pceHeapAlloc` already guarantees.
const HEAP_ALIGN: usize = 4;

/// Zero-sized global allocator backed by the kernel heap. See the module docs.
pub struct PceHeap;

unsafe impl GlobalAlloc for PceHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= HEAP_ALIGN {
            return ffi::pceHeapAlloc(layout.size() as _) as *mut u8;
        }

        // Over-aligned: reserve `align` slack to reach an aligned address plus one
        // usize below it to store the true base for `dealloc`.
        let align = layout.align();
        let overhead = align + size_of::<usize>();
        let base = ffi::pceHeapAlloc((layout.size() + overhead) as _) as *mut u8;
        if base.is_null() {
            return ptr::null_mut();
        }
        let base_addr = base as usize;
        let user_addr = (base_addr + size_of::<usize>() + align - 1) & !(align - 1);
        let user = user_addr as *mut u8;
        (user as *mut usize).sub(1).write_unaligned(base_addr);
        user
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= HEAP_ALIGN {
            ffi::pceHeapFree(ptr as *mut c_void);
        } else {
            let base = (ptr as *mut usize).sub(1).read_unaligned();
            ffi::pceHeapFree(base as *mut c_void);
        }
    }

    // `realloc` / `alloc_zeroed` use the GlobalAlloc defaults: realloc =
    // alloc + copy + dealloc (pceHeapRealloc can't relocate, so it is unused);
    // alloc_zeroed = alloc + write_bytes(0).
}

/// Largest currently-allocatable block, in bytes (`pceHeapGetMaxFreeSize`).
#[inline]
pub fn max_free_size() -> usize {
    unsafe { ffi::pceHeapGetMaxFreeSize() as usize }
}
