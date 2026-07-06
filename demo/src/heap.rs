//! Global allocator backed by the P/ECE kernel heap (`pceHeapAlloc` /
//! `pceHeapFree`).
//!
//! Why the kernel heap rather than libc `malloc`: `pceHeapAlloc` bounds-checks
//! against `sram_end` and returns NULL on exhaustion (→ Rust's alloc-error path),
//! whereas libc's `sbrk`-backed malloc has no upper bound and silently corrupts
//! kernel state on overrun. C/C++ were forced onto libc malloc because a correct
//! C `realloc()` must relocate and `pceHeapRealloc` cannot; Rust's `GlobalAlloc`
//! only needs `alloc`/`dealloc` and composes `realloc` itself, so it is free to
//! use the safer kernel heap. As long as this app never calls libc `malloc`, the
//! kernel heap owns the whole `[bss_end, sram_end)` region.
//!
//! `pceHeapAlloc` guarantees only 4-byte alignment. Requests with a larger
//! alignment are satisfied by over-allocating and stashing the real base pointer
//! just below the returned (aligned) pointer, to be recovered on free.

use core::alloc::{GlobalAlloc, Layout};
use core::mem::size_of;
use core::ptr;

extern "C" {
    /// `void *pceHeapAlloc(unsigned long size)` — 4-byte aligned, NULL on failure.
    fn pceHeapAlloc(size: usize) -> *mut u8;
    /// `int pceHeapFree(void *memp)` — 0 on success (return value ignored here).
    fn pceHeapFree(memp: *mut u8) -> i32;
}

/// Alignment `pceHeapAlloc` already guarantees.
const HEAP_ALIGN: usize = 4;

struct PceHeap;

unsafe impl GlobalAlloc for PceHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if layout.align() <= HEAP_ALIGN {
            return pceHeapAlloc(layout.size());
        }

        // Over-aligned: reserve `align` slack to reach an aligned address plus one
        // usize below it to store the true base for `dealloc`.
        let align = layout.align();
        let overhead = align + size_of::<usize>();
        let base = pceHeapAlloc(layout.size() + overhead);
        if base.is_null() {
            return ptr::null_mut();
        }
        let base_addr = base as usize;
        // First aligned address that leaves room for the stored base pointer.
        let user_addr = (base_addr + size_of::<usize>() + align - 1) & !(align - 1);
        let user = user_addr as *mut u8;
        // Stash the real base just below the user pointer.
        (user as *mut usize).sub(1).write_unaligned(base_addr);
        user
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if layout.align() <= HEAP_ALIGN {
            pceHeapFree(ptr);
        } else {
            // Recover the real base stashed just below the user pointer.
            let base = (ptr as *mut usize).sub(1).read_unaligned();
            pceHeapFree(base as *mut u8);
        }
    }

    // `realloc` and `alloc_zeroed` use the GlobalAlloc defaults: realloc =
    // alloc + copy + dealloc (pceHeapRealloc can't relocate, so we don't use it);
    // alloc_zeroed = alloc + write_bytes(0).
}

#[global_allocator]
static ALLOCATOR: PceHeap = PceHeap;
