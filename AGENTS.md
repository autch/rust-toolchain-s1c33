# AGENTS.md — Rust for the EPSON S1C33000 (Aquaplus P/ECE)

## What this project is

This is a fork of the Rust compiler (`rustc` 1.96.0) being taught to target the
**EPSON S1C33000** 32-bit RISC core, specifically the **Aquaplus P/ECE**
(S1C33209 SoC). It is paired with an out-of-tree LLVM fork that provides the
S1C33 code generator.

**Immediate goal — ACHIEVED:** build P/ECE-loadable applications (`.pex`) from Rust
alone (plus the `llvm-c33` tools), ABI-compatible with the P/ECE kernel and SDK.
A pure-Rust app (`demo/`) now builds, links against the language-agnostic sysroot,
packs to `.pex`, and renders "Hello from Rust!" on both piece-emu and real hardware.

## The two repositories

This document lives at the root of the **toolchain wrapper repo**, which carries
the rustc fork as its `rust/` git submodule. rustc source paths and build commands
below are relative to that submodule root (`rust/`) — `cd rust` first.

| Path | What it is |
|---|---|
| `.` (repo root) | This repo — the toolchain wrapper. Ties the pieces together. |
| `rust/` | The rustc fork submodule (`git@github.com:autch/rust-s1c33.git`) with the `s1c33-none-piece` target. |
| `demo/` | Minimal pure-Rust P/ECE app (staticlib) — the end-to-end ABI proof. See "Building the demo app" below. |
| `llvm-c33/` | LLVM 22.1.1 fork with the S1C33 backend (Phase 6 complete: builds real P/ECE C apps that run on hardware). Has its own `CLAUDE.md` and `DESIGN_SPEC.md`. **Not a submodule** — it is shared with other projects, so it is an in-tree **symlink** to your own checkout (gitignored). Create it once: `ln -s ../llvm-c33 llvm-c33`. |

`llvm-c33/` is a repo-root-relative path (via the symlink); build commands run from
`rust/` or `demo/`, so from there reference it as `../llvm-c33/`.

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
- The target pins **`cpu = "s1c33209"`** (P/ECE SoC → hardware multiplier, so `i32`
  MUL lowers to `mlt.w` inline instead of a `__mulsi3` libcall that nothing provides)
  and **`min_global_align = 32`** (gcc33/clang parity — without it a `static [u8; N]`
  can land at an odd address and the kernel's word/halfword accesses trap). See
  "gcc33 / clang ABI parity" below.
- **End-to-end ABI proof COMPLETE**: the `demo/` app builds → links → packs → runs.
  Renders "Hello from Rust!" on piece-emu and real hardware. See "Building the demo
  app" for the exact recipe.
- Frontend-parity audit against clang's `S1C33TargetInfo` found no remaining gaps
  (data layout byte-identical, char signedness both signed, all widths/alignments
  match). See "gcc33 / clang ABI parity".

Later / optional: `alloc` + global allocator (pceHeap), fuller pceapi bindings,
`.cargo/config` + target link-args + a `build.rs`/script or cargo runner so a single
`cargo build`/`cargo run` produces (and packs/runs) the `.pex` directly. The demo
currently links and packs via explicit commands (below).

## Build workflow (READ THIS — several steps are non-obvious)

All build steps below run from inside the `rust/` submodule (`cd rust` from the
repo root).

`rust/bootstrap.toml` already points at the custom LLVM and sets:
```
llvm.download-ci-llvm = false
llvm.optimize = false
[target.x86_64-unknown-linux-gnu]
llvm-config = "../llvm-c33/build/bin/llvm-config"   # via the in-tree symlink; an absolute path also works
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
  `ninja -C ../llvm-c33/build llc`
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
rust-src is already present in the stage1 sysroot (`lib/rustlib/src/rust`) — but
note it is a **symlink to this repo's absolute path**, so if the repo is moved it
dangles (build-std then errors "does not exist, ... rustup component add rust-src").
A `./x.py` run regenerates it; or repoint it by hand.

### Building the demo app (`demo/`, a pure-Rust `.pex`)
Runs from `demo/` (the wrapper repo root), **not** `rust/`. `demo/.cargo/config.toml`
sets `build-std=["core"]` and the target, so no extra flags are needed — the CPU
(`s1c33209`) and min-global-align are baked into the target spec.

1. **Build the staticlib** (crate-type `staticlib`, so cargo never invokes the
   linker — we link `libdemo.a` by hand):
   ```
   RUSTC=<rust>/build/x86_64-unknown-linux-gnu/stage1/bin/rustc RUSTC_BOOTSTRAP=1 \
     <rust>/build/x86_64-unknown-linux-gnu/stage0/bin/cargo build --release
   # → target/s1c33-none-piece/release/libdemo.a
   ```
2. **Link** against the language-agnostic sysroot. `--gc-sections` is ESSENTIAL: it
   drops unreferenced `core` code (a hello-world's `.text` goes 162 KB → <1 KB);
   piece.ld `KEEP`s crt0.o so `pceAppHead` survives GC.
   ```
   ld.lld --gc-sections -T ../llvm-c33/tools/piece.ld -o demo.elf \
     ../llvm-c33/sysroot/s1c33-none-piece/lib/crt0.o \
     --start-group target/s1c33-none-piece/release/libdemo.a \
       -L../llvm-c33/sysroot/s1c33-none-piece/lib -lpceapi -lpceshim -lc -lm --end-group
   ```
3. **Pack**: `../llvm-c33/tools/ppack/ppack -e -N"RustDemo" demo.elf -odemo.pex`

Non-obvious points (all cost real debugging time — don't relearn them):
- The app must define `pceAppInit`/`pceAppProc`/`pceAppExit`/**`pceAppNotify`** (all
  four are undefined symbols in crt0.o) + a `#[panic_handler]` + a spinning `_exit`
  (compiler_builtins' cold paths reach `abort` → picolibc `abort` → `_exit`, which
  the picolibc sysroot does not provide; mirror newlib's `sys/s1c33/_exit.c`).
- crt0 clears BSS before `pceAppInit`, so zero-initialised statics need no memset.
- **Global alignment**: relies on the target's `min_global_align = 32`; without it a
  bare `static [u8; N]` (align 1) lands at an odd address and the kernel's LCD
  transfer word-reads it → misalignment trap. (See "gcc33 / clang ABI parity".)
- **Running headlessly**: swap the `.pex` in as `startup.pex` with
  `../llvm-c33/piece-emu/build-release/tools/pfar <img>.pfi -a startup.pex` (the kernel
  auto-runs `startup.pex`), then boot
  `../llvm-c33/piece-emu/build-src/piece-emu-headless-system <img>.pfi --script run.txt`;
  the script's `snapshot` command writes PNGs. Kernel images: `piece-emu/images/`.

### Memory / linker constraint
This machine **swaps if parallel linkers pile up**. The LLVM build cache is set to
`LLVM_PARALLEL_LINK_JOBS=1` (plus `LLVM_OPTIMIZED_TABLEGEN=ON`,
`LLVM_BUILD_TESTS=OFF`, `LLVM_INCLUDE_TESTS=OFF`). Keep it that way; don't run
many heavy links at once.

## Where the s1c33 support lives in rustc

| File | What |
|---|---|
| `compiler/rustc_target/src/spec/targets/s1c33_none_piece.rs` | The builtin target spec. Also pins `cpu = "s1c33209"` (HWMul) and `min_global_align = 32` (gcc33 parity — see below). |
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

## Debugging a backend crash during `core` build (fast loop, no rustc rebuild)

rustc suppresses LLVM stack traces. Instead:
1. `--emit=llvm-ir` via `CARGO_ENCODED_RUSTFLAGS` — the `.ll` is written before the
   backend crash. Find it under `target/s1c33-none-piece/release/deps/*.ll`.
2. Run `../llvm-c33/build/bin/llc -mtriple=s1c33-none-piece -O3 the.ll -o /dev/null`
   → real stack trace pointing at the failing pass / operation.
3. If needed, add a temporary `dbgs()`/message in the generic LLVM file to print
   opcodes/types, rebuild only `llc`, re-run. **Revert such diagnostics before
   committing** (they touch shared LLVM files).

## Relevant commits

- rust `71c0c75c7b3` — Add s1c33-none-piece target.
- rust `e1079df7579` — mark f16/f128 unreliable for s1c33.
- rust `92958598444` — pin cpu=s1c33209 + min_global_align=32 (gcc33 ABI parity).
- llvm submodule `f6e7f2d31f56` — i128 + wide libcall support.
- llvm-c33 parent `a2ccd4f` — bump llvm submodule.
- wrapper repo `61ddd43` — initial commit (rust submodule + `demo/` + this file).
