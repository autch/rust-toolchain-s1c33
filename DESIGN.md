# DESIGN.md — Rust for the EPSON S1C33000 (Aquaplus P/ECE)

Goals, design decisions, and roadmap for the Rust-on-S1C33 toolchain. Operational
detail (repo layout, build/test workflow, where the code lives) is in
[`AGENTS.md`](AGENTS.md).

## Goal

Teach a fork of the Rust compiler (`rustc` 1.96.0) to target the **EPSON S1C33000**
32-bit RISC core, specifically the **Aquaplus P/ECE** (S1C33209 SoC), paired with an
out-of-tree LLVM fork that provides the S1C33 code generator.

**Immediate goal — ACHIEVED:** build P/ECE-loadable applications (`.pex`) from Rust
alone (plus the `llvm-c33` tools), ABI-compatible with the P/ECE kernel and SDK.
A pure-Rust app (`demo/`) builds, links against the language-agnostic sysroot, packs
to `.pex`, and runs on both piece-emu and real hardware — using a global allocator on
the kernel heap, the `pceapi` bindings, and drawing text + live SYSTEMINFO.

## Status & roadmap

Done:
- `s1c33-none-piece` is a **builtin target** (`--target s1c33-none-piece`), tier-3,
  `no_std`, `panic=abort`, `EM_SE_C33` (ELF machine 107).
- Calling convention (`S5U1C33000C` ABI) implemented and verified at IR/asm level,
  including the §3.5 single-element-struct-in-register quirk.
- `core` and `compiler_builtins` **cross-compile** via `-Zbuild-std=core`; a real
  `#![no_std]` crate builds against them.
- i128 and the wide runtime libcalls `core` needs are handled in the backend.
- The target pins **`cpu = "s1c33209"`** (P/ECE SoC → hardware multiplier, so `i32`
  MUL lowers to `mlt.w` inline instead of a `__mulsi3` libcall that nothing provides)
  and **`min_global_align = 32`** (gcc33/clang parity — without it a `static [u8; N]`
  can land at an odd address and the kernel's word/halfword accesses trap). See
  "gcc33 / clang ABI parity" below.
- **End-to-end ABI proof COMPLETE**: the `demo/` app builds → links → packs → runs
  on piece-emu and real hardware. See AGENTS.md → "Building the demo app".
- Frontend-parity audit against clang's `S1C33TargetInfo` found no remaining gaps
  (data layout byte-identical, char signedness both signed, all widths/alignments
  match). See "gcc33 / clang ABI parity".
- **`alloc` works**: `heap::PceHeap` (in `pceapi`) is a `GlobalAlloc` on the kernel
  heap (`pceHeapAlloc`/`Free`); `Vec`/`String`/`format!` run on device. Rust uses the
  kernel heap — not libc malloc that C/C++ were forced onto — because it composes
  `realloc` itself and the kernel heap bounds-checks (NULL on OOM vs sbrk corruption).
- **`pceapi` binds the full piece.h + draw.h API** (all structs `#[repr(C)]`, all
  externs, constants) with safe wrappers for every module (pad/lcd/font/draw/app/cpu/
  heap/system/time/power/file/wave/timer/flash/ir/usb/vector/debug); only the variadic
  `pceFontPrintf`/`pcesprintf` stay raw-ffi (no generic safe wrapper). draw.h adds the
  graphics primitives (point/line/paint) and object blits (DRAW_OBJECT passed by value
  — the byval-struct-on-stack ABI). Bind only symbols with a KSNO_/KSNO2_ kernel entry
  (vector.h) AND a libpceapi.a stub.
- **ABI runtime-verified** (not just IR): c-variadic (caller side — via the kernel's
  `pceFontPrintf` and a clang C fixture) and struct-by-value (multi-field on stack +
  §3.5 single 8/16/32-bit element in register) both confirmed on piece-emu. Now
  regression-guarded by the `abitest/` semihosting suite (AGENTS.md → "ABI unit tests").

Later / optional: `.cargo/config` + target link-args + a `build.rs`/script or cargo
runner so a single `cargo build`/`cargo run` produces (and packs/runs) the `.pex`
directly. The demo currently links and packs via explicit commands (AGENTS.md →
"Building the demo app"). (`pceapi` wrappers are otherwise complete.)

## ABI is authoritative in the LLVM fork, not here

The `S5U1C33000C` calling convention is documented in `llvm-c33/DESIGN_SPEC.md`
§2.2/§3 and `llvm-c33/docs/errata.md`, and implemented in
`llvm-c33/llvm/clang/lib/CodeGen/Targets/S1C33.cpp` (`S1C33ABIInfo`). rustc's
`callconv/s1c33.rs` **mirrors that**. Key points:
- Args R12→R15 (overflow to stack); return R10 (R11:R10 for 64-bit); struct return
  via sret pointer in R12 (no single-element exception for returns).
- Struct args go entirely on the stack, EXCEPT a single integer/enum/pointer
  element of exactly 8/16/32 bits, which is passed in a register (coerced to that
  integer, marked `inreg`; the backend high-bit-packs the 8/16-bit forms).
- When changing the callconv, cross-check against DESIGN_SPEC §3 and
  `S1C33ABIInfo` — do not blindly copy another backend (the m68k-derived first cut
  missed §3.5).
- The mirror is **runtime-verified** against clang, not just at the IR level:
  `abitest/` passes Rust-declared `#[repr(C)]` structs (multi-field and the §3.5
  single-element forms) and varargs to clang-built C fixtures and checks the
  results. Re-run `./abitest/run.sh` after touching `callconv/s1c33.rs`.

## gcc33 / clang ABI parity (frontend settings rustc must mirror)

gcc33 is the reference ABI compiler; clang's `S1C33TargetInfo`
(`llvm-c33/llvm/clang/lib/Basic/Targets/S1C33.{h,cpp}`) encodes the frontend ABI
decisions. **Backend** quirks (instruction encodings, delay slots, PC-rel
relocations, PSR liveness, 64-bit arg/emu-lib conventions, variadic ABI,
memcpy→byte-ops) live in the LLVM S1C33 backend and apply to clang AND rustc for
free. But **frontend** settings live in clang's TargetInfo and have to be set
independently in the rustc target spec — miss one and Rust steps in the same rut.
Audited; parity as of now:

- **Data layout string** — byte-identical to clang's `resetDataLayout(...)`. This is
  where `i64:32`, `f64:32` (64-bit types only need 4-byte alignment), `a:0:32`, and
  `S32` come from. If you touch it, keep it identical to clang's.
- **`min_global_align = 32`** — clang's `MinGlobalAlign=32`. All globals ≥4-byte
  aligned. `a:0:32` only covers *aggregate* (struct) alignment, NOT arrays, so it
  does **not** substitute for this. (Documented in `llvm-c33/docs/errata.md`
  "MinGlobalAlign=32".)
- **`cpu = "s1c33209"`** — clang gets HWMul from the backend's empty-CPU default;
  rustc always passes a CPU, so an explicit `s1c33209` is needed or MUL → `__mulsi3`.
- **char signedness** — clang leaves `CharIsSigned` at its signed default; Rust's
  `c_char` is `i8` (signed) for s1c33 (not in core's unsigned-char arch list). Match.
- Int/long/pointer widths, TLS-off, atomics-off: all match (widths via `c_int_width`
  / `core::ffi`, layout via data-layout, `max_atomic_width = 0`).

## Exotic wide types: i128 and f16/f128

i128 is **not** an S5U1C33000C ABI type, but Rust's `core` uses it, so it must not
crash — it degrades to memory/libcalls. The backend handles it:
`CanLowerReturn` demotes oversized returns to sret; the immediate cost model is
width-safe (`isSignedIntN`); and i128 mul/div/rem/shift, float↔i128, fmod, fma,
and MULO (`__mulodi4`/`__muloti4`) libcalls are registered. The S1C33 backend
starts from an **empty libcall table** and registers each libcall explicitly, so a
new operation that needs a runtime call must be added there.

f16/f128 are marked **unreliable** for s1c33 in `llvm_util.rs` (no such types on
this hardware; the soft-float lib only covers f32/f64). This keeps `core` from
emitting them. This is a Rust-side decision and does NOT affect clang.
