//! Bindings to the Aquaplus P/ECE kernel API (EPSON S1C33209).
//!
//! Layered as a thin binding for the `s1c33-none-piece` target:
//! - [`ffi`] — raw `extern "C"` declarations matching `sdk/include/piece.h`,
//!   plus the kernel API constants. The symbols live in `libpceapi.a`.
//! - Safe modules ([`pad`], [`lcd`], [`font`], [`app`], [`cpu`], [`heap`]) —
//!   idiomatic wrappers over the everyday surface.
//!
//! This is a *core subset*: input, display, fonts, app lifecycle, CPU speed and
//! the kernel heap. File/Wave/IR/USB/Power/Time and their structs are not bound
//! yet (declare via [`ffi`] as needed, or extend this crate).
//!
//! `no_std`. The `alloc` feature adds allocating conveniences.

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod app;
pub mod cpu;
pub mod ffi;
pub mod font;
pub mod heap;
pub mod lcd;
pub mod pad;

pub use pad::{Button, Pad};
