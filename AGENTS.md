# AGENTS.md — Rust for the EPSON S1C33000 (Aquaplus P/ECE)

## What this project is

This is a fork of the Rust compiler (`rustc` 1.96.0) being taught to target the
**EPSON S1C33000** 32-bit RISC core, specifically the **Aquaplus P/ECE**
(S1C33209 SoC). It is paired with an out-of-tree LLVM fork that provides the
S1C33 code generator.

**Immediate goal:** build P/ECE-loadable applications (`.pex`) from Rust alone
(plus the `llvm-c33` tools), ABI-compatible with the P/ECE kernel and SDK.

## The two repositories

This document lives at the root of the **toolchain wrapper repo**, which carries
the rustc fork as its `rust/` git submodule. rustc source paths and build commands
below are relative to that submodule root (`rust/`) — `cd rust` first.

| Path | What it is |
|---|---|
| `.` (repo root) | This repo — the toolchain wrapper. Ties the pieces together. |
| `rust/` | The rustc fork submodule (`git@github.com:autch/rust-s1c33.git`) with the `s1c33-none-piece` target. |
| `/home/autch/src/llvm-c33` | LLVM 22.1.1 fork with the S1C33 backend (Phase 6 complete: builds real P/ECE C apps that run on hardware). Sibling of this repo; has its own `CLAUDE.md` and `DESIGN_SPEC.md`. |

rustc links against the LLVM fork via `llvm-config` (see `rust/bootstrap.toml`).
The LLVM backend lives in `llvm-c33/llvm/llvm/lib/Target/S1C33/`
(note: `llvm` is the LLVM monorepo git submodule inside `llvm-c33`,
hence the doubled `llvm/llvm`).

**Language rule (both repos):** all code, comments, and commit messages are
**English only**. This applies to the Rust side too.

## Current status

Done:
- `s1c33-none-piece` is a **builtin target** (`--target s1c33-none-piece`), tier-3,
  `no_std`, `panic=abort`, `EM_SE_C33` (ELF machine 107).
- Calling convention (`S5U1C33000C` ABI) implemented and verified at IR/asm level,
  including the §3.5 single-element-struct-in-register quirk.
- `core` and `compiler_builtins` **cross-compile** via `-Zbuild-std=core`; a real
  `#![no_std]` crate builds against them.
- i128 and the wide runtime libcalls `core` needs are handled in the backend.

Remaining toward the goal (integration/glue, the hard compiler work is done):
1. Rust app skeleton: `#![no_main]` + `extern "C"` `pceAppInit`/`pceAppProc`/
   `pceAppExit` + `#[panic_handler]`. P/ECE apps have no `main`; crt0 calls these
   callbacks by symbol name.
2. `pceapi` bindings: `extern "C"` declarations for the kernel APIs used
   (`pceLCDTrans`, `pcePadGet`, `pceFontPutStr`, ...). Hand-write for the demo.
3. **Link recipe (main piece):** link the Rust `.o` with the existing sysroot
   artifacts — `ld.lld crt0.o crti.o <rust.o> --start-group -lpceapi -lpceshim
   -lc -lm --end-group -T piece.ld` → `.elf`. Reuse
   `llvm-c33/sysroot/s1c33-none-piece/lib/` (crt0.o, piece.ld,
   libpceapi.a, ... are language-agnostic and already built). Then
   `ppack app.elf` → `app.pex`.
4. Resolve runtime symbols (memcpy/memset from compiler_builtins `mem` feature vs
   SDK libs; watch for duplicate-symbol conflicts).
5. Verify end-to-end on `llvm-c33/piece-emu` or hardware (this is the
   ABI proof).

Later / optional: `alloc` + global allocator (pceHeap), fuller pceapi bindings,
`.cargo/config` + target link-args so `cargo build` produces the `.elf` directly,
ppack as a cargo runner.

## Build workflow (READ THIS — several steps are non-obvious)

All build steps below run from inside the `rust/` submodule (`cd rust` from the
repo root).

`rust/bootstrap.toml` already points at the custom LLVM and sets:
```
llvm.download-ci-llvm = false
llvm.optimize = false
[target.x86_64-unknown-linux-gnu]
llvm-config = "/home/autch/src/llvm-c33/build/bin/llvm-config"
[target.s1c33-none-piece]
optimized-compiler-builtins = false   # cc-rs doesn't know s1c33; use pure-Rust builtins
```

### Building rustc
```
./x.py build --stage 1 library        # builds stage1 rustc AND host std
```
- Use `library`, not `compiler/rustc`, so **host std stays consistent**. Build
  scripts (e.g. compiler_builtins') compile for the HOST with the stage1 rustc; if
  host std is missing you get `E0463: can't find crate for std`. Any time you
  rebuild rustc, rebuild `library` too.
- After changing an LLVM `.cpp` (backend),
  `ninja -C /home/autch/src/llvm-c33/build llc`
  rebuilds `libLLVMS1C33CodeGen.a`; then `touch compiler/rustc_llvm/build.rs` and
  rebuild so rustc relinks against the new lib.

### Building a target crate (`core` etc.) — build-std, NOT x.py
`./x.py build --target s1c33-none-piece library/core` **fails**: bootstrap's
`cc_detect` runs the `cc` crate for the target and cc-rs errors with
"unknown architecture s1c33". Use build-std with cargo + the stage1 rustc:
```
RUSTC=$(pwd)/build/x86_64-unknown-linux-gnu/stage1/bin/rustc RUSTC_BOOTSTRAP=1 \
  $(pwd)/build/x86_64-unknown-linux-gnu/stage0/bin/cargo build \
  -Z build-std=core --target s1c33-none-piece --release
```
rust-src is already present in the stage1 sysroot (`lib/rustlib/src/rust`).

### Memory / linker constraint
This machine **swaps if parallel linkers pile up**. The LLVM build cache is set to
`LLVM_PARALLEL_LINK_JOBS=1` (plus `LLVM_OPTIMIZED_TABLEGEN=ON`,
`LLVM_BUILD_TESTS=OFF`, `LLVM_INCLUDE_TESTS=OFF`). Keep it that way; don't run
many heavy links at once.

## Where the s1c33 support lives in rustc

| File | What |
|---|---|
| `compiler/rustc_target/src/spec/targets/s1c33_none_piece.rs` | The builtin target spec. |
| `compiler/rustc_target/src/spec/mod.rs` | `Arch::S1C33`, `object_architecture` (returns `None` — the `object` crate has no S1C33 arch, so metadata uses the fallback), TARGETS registration, exhaustive `Arch` matches. |
| `compiler/rustc_target/src/callconv/s1c33.rs` | The `S5U1C33000C` ABI lowering (mirrors clang's `S1C33ABIInfo`). |
| `compiler/rustc_target/src/callconv/mod.rs` | Dispatch to `s1c33::compute_abi_info`. |
| `compiler/rustc_target/src/target_features.rs`, `asm/mod.rs` | S1C33 arms in exhaustive matches. |
| `compiler/rustc_span/src/symbol.rs` | `s1c33` symbol. |
| `compiler/rustc_llvm/build.rs`, `src/lib.rs` | Enable + initialize the S1C33 LLVM target. |
| `compiler/rustc_codegen_llvm/src/va_arg.rs` | S1C33 c-variadic → LLVM backend. |
| `compiler/rustc_codegen_llvm/src/llvm_util.rs` | `has_reliable_f16`/`f128` = false for s1c33 (see below). |

When you add a first-class `Arch` variant, the compiler flags every non-exhaustive
`match` over `Arch` — fix each; that's how the full set above was found.

## ABI is authoritative in the LLVM fork, not here

The `S5U1C33000C` calling convention is documented in
`llvm-c33/DESIGN_SPEC.md` §2.2/§3 and
`llvm-c33/docs/errata.md`, and implemented in
`llvm-c33/llvm/clang/lib/CodeGen/Targets/S1C33.cpp`
(`S1C33ABIInfo`). rustc's
`callconv/s1c33.rs` **mirrors that**. Key points:
- Args R12→R15 (overflow to stack); return R10 (R11:R10 for 64-bit); struct return
  via sret pointer in R12 (no single-element exception for returns).
- Struct args go entirely on the stack, EXCEPT a single integer/enum/pointer
  element of exactly 8/16/32 bits, which is passed in a register (coerced to that
  integer, marked `inreg`; the backend high-bit-packs the 8/16-bit forms).
- When changing the callconv, cross-check against DESIGN_SPEC §3 and
  `S1C33ABIInfo` — do not blindly copy another backend (the m68k-derived first cut
  missed §3.5).

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

## Debugging a backend crash during `core` build (fast loop, no rustc rebuild)

rustc suppresses LLVM stack traces. Instead:
1. `--emit=llvm-ir` via `CARGO_ENCODED_RUSTFLAGS` — the `.ll` is written before the
   backend crash. Find it under `target/s1c33-none-piece/release/deps/*.ll`.
2. Run `llvm-c33/build/bin/llc -mtriple=s1c33-none-piece -O3 the.ll -o /dev/null`
   → real stack trace pointing at the failing pass / operation.
3. If needed, add a temporary `dbgs()`/message in the generic LLVM file to print
   opcodes/types, rebuild only `llc`, re-run. **Revert such diagnostics before
   committing** (they touch shared LLVM files).

## Relevant commits

- rust `71c0c75c7b3` — Add s1c33-none-piece target.
- rust `e1079df7579` — mark f16/f128 unreliable for s1c33.
- llvm submodule `f6e7f2d31f56` — i128 + wide libcall support.
- llvm-c33 parent `a2ccd4f` — bump llvm submodule.
