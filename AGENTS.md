# AGENTS.md — Rust for the EPSON S1C33000 (Aquaplus P/ECE)

## What this project is

A fork of the Rust compiler (`rustc` 1.96.0) taught to target the **EPSON S1C33000**
32-bit RISC core — the **Aquaplus P/ECE** (S1C33209 SoC) — paired with an out-of-tree
LLVM fork that provides the S1C33 code generator. Pure-Rust `.pex` apps build and run
on hardware today.

> **Goals, design decisions, and roadmap live in [`DESIGN.md`](DESIGN.md).**
> This file (AGENTS.md) is the operational guide: repo layout, the build/test
> workflow, and where the code lives.

## The two repositories

This document lives at the root of the **toolchain wrapper repo**, which carries
the rustc fork as its `rust/` git submodule. rustc source paths and build commands
below are relative to that submodule root (`rust/`) — `cd rust` first.

| Path | What it is |
|---|---|
| `.` (repo root) | This repo — the toolchain wrapper. Ties the pieces together. |
| `rust/` | The rustc fork submodule (`git@github.com:autch/rust-s1c33.git`) with the `s1c33-none-piece` target. |
| `pceapi/` | Bindings crate for the P/ECE kernel API (`ffi` = raw `extern "C"` for all of piece.h + constants; safe wrappers `pad`/`lcd`/`font`/`app`/`cpu`/`heap`/`system`/`time`/`power`/`file`/`wave`/`timer`/`flash`/`ir`/`usb`/`vector`/`debug`). Includes `heap::PceHeap` (the kernel-heap `GlobalAlloc`). Only the variadic `pceFontPrintf`/`pcesprintf` stay raw-ffi (no generic safe wrapper). |
| `demo/` | Minimal pure-Rust P/ECE app (staticlib) — the end-to-end ABI proof. Uses `pceapi`, installs `PceHeap`, exercises `alloc`, and displays SYSTEMINFO. See "Building the demo app". |
| `abitest/` | Bare-metal ABI unit tests run under piece-emu via semihosting (exit code = pass/fail). `./abitest/run.sh`. See "ABI unit tests". |
| `llvm-c33/` | LLVM 22.1.1 fork with the S1C33 backend (Phase 6 complete: builds real P/ECE C apps that run on hardware). Has its own `CLAUDE.md` and `DESIGN_SPEC.md`. **Not a submodule** — it is shared with other projects, so it is an in-tree **symlink** to your own checkout (gitignored). Create it once: `ln -s ../llvm-c33 llvm-c33`. |

`llvm-c33/` is a repo-root-relative path (via the symlink); build commands run from
`rust/` or `demo/`, so from there reference it as `../llvm-c33/`.

rustc links against the LLVM fork via `llvm-config` (see `rust/bootstrap.toml`).
The LLVM backend lives in `llvm-c33/llvm/llvm/lib/Target/S1C33/`
(note: `llvm` is the LLVM monorepo git submodule inside `llvm-c33`,
hence the doubled `llvm/llvm`).

**Language rule (both repos):** all code, comments, and commit messages are
**English only**. This applies to the Rust side too.

Current status and roadmap: see [`DESIGN.md`](DESIGN.md) → "Status & roadmap".

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
sets `build-std=["core", "alloc"]` and the target, so no extra flags are needed — the
CPU (`s1c33209`) and min-global-align are baked into the target spec. `demo` depends
on `pceapi` (`path = "../pceapi"`, `features = ["alloc"]`); cargo builds it too.

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
  transfer word-reads it → misalignment trap. (See DESIGN.md → "gcc33 / clang ABI
  parity".)
- **Running headlessly**: swap the `.pex` in as `startup.pex` with
  `../llvm-c33/piece-emu/build-release/tools/pfar <img>.pfi -a startup.pex` (the kernel
  auto-runs `startup.pex`), then boot
  `../llvm-c33/piece-emu/build-src/piece-emu-headless-system <img>.pfi --script run.txt`;
  the script's `snapshot` command writes PNGs. Kernel images: `piece-emu/images/`.

### ABI unit tests (`abitest/`, semihosting — fast, no kernel)
`abitest/run.sh` builds a bare-metal Rust test and runs it under `piece-emu`,
reporting pass/fail through the emulator's **semihosting** ports (`0x060000`;
`TEST_RESULT` at `0x060008`, 0 = PASS). The emulator turns that into its exit code,
so it is a scriptable/CI check — much faster than eyeballing an LCD snapshot.
- No P/ECE kernel: reuses the bare-metal `crt0.s`/`crt_init.c`/`iram.ld` from
  `llvm-c33/piece-emu/src/tests/bare_metal` (`_start` → `_start_c` → `main()`, whose
  return is written to TEST_RESULT). Code runs in IRAM; keep tests alloc-free/small.
- `abitest/src/lib.rs` is a `#[no_mangle] extern "C" fn main() -> i32` with
  `check!(expr, code)` assertions (distinct code per check → `[FAIL] code=N`).
- Cross-language ABI fixtures live in `abitest/csrc/abi_test.c`, built by clang
  (`--target=s1c33-none-elf`); calling them from Rust checks the calling convention
  against clang at runtime. Add a check + (if needed) a C fixture + `extern` decl.
- Kernel APIs (`pceapi`) are NOT available here (no kernel) — test those in `demo/`.

### Memory / linker constraint
This machine **swaps if parallel linkers pile up**. The LLVM build cache is set to
`LLVM_PARALLEL_LINK_JOBS=1` (plus `LLVM_OPTIMIZED_TABLEGEN=ON`,
`LLVM_BUILD_TESTS=OFF`, `LLVM_INCLUDE_TESTS=OFF`). Keep it that way; don't run
many heavy links at once.

## Where the s1c33 support lives in rustc

| File | What |
|---|---|
| `compiler/rustc_target/src/spec/targets/s1c33_none_piece.rs` | The builtin target spec. Also pins `cpu = "s1c33209"` (HWMul) and `min_global_align = 32` (gcc33 parity — see DESIGN.md). |
| `compiler/rustc_target/src/spec/mod.rs` | `Arch::S1C33`, `object_architecture` (returns `None` — the `object` crate has no S1C33 arch, so metadata uses the fallback), TARGETS registration, exhaustive `Arch` matches. |
| `compiler/rustc_target/src/callconv/s1c33.rs` | The `S5U1C33000C` ABI lowering (mirrors clang's `S1C33ABIInfo`). |
| `compiler/rustc_target/src/callconv/mod.rs` | Dispatch to `s1c33::compute_abi_info`. |
| `compiler/rustc_target/src/target_features.rs`, `asm/mod.rs` | S1C33 arms in exhaustive matches. |
| `compiler/rustc_span/src/symbol.rs` | `s1c33` symbol. |
| `compiler/rustc_llvm/build.rs`, `src/lib.rs` | Enable + initialize the S1C33 LLVM target. |
| `compiler/rustc_codegen_llvm/src/va_arg.rs` | S1C33 c-variadic → LLVM backend. |
| `compiler/rustc_codegen_llvm/src/llvm_util.rs` | `has_reliable_f16`/`f128` = false for s1c33 (see DESIGN.md → "Exotic wide types"). |

When you add a first-class `Arch` variant, the compiler flags every non-exhaustive
`match` over `Arch` — fix each; that's how the full set above was found.

The **ABI** (calling convention, gcc33/clang frontend parity, i128/f16/f128
handling) is a set of design decisions with rules for changing them safely — see
[`DESIGN.md`](DESIGN.md). In particular, `callconv/s1c33.rs` mirrors clang's
`S1C33ABIInfo`; after touching it, re-run `./abitest/run.sh` (the mirror is
runtime-verified there).

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

rust submodule (`autch/rust-s1c33`):
- `71c0c75c7b3` — Add s1c33-none-piece target.
- `e1079df7579` — mark f16/f128 unreliable for s1c33.
- `92958598444` — pin cpu=s1c33209 + min_global_align=32 (gcc33 ABI parity).

llvm-c33: llvm submodule `f6e7f2d31f56` (i128 + wide libcalls); parent `a2ccd4f`
(bump llvm submodule).

wrapper repo (this repo, `main`):
- `61ddd43` — initial commit (rust submodule + `demo/` + this file).
- pceHeap global allocator + `alloc`; `pceapi` bindings crate (core subset, then
  full piece.h API + system/time/power/file wrappers); c-variadic + struct ABI
  runtime checks in `demo/`; `abitest/` semihosting unit-test harness; demo
  displays SYSTEMINFO. (`git log` for the exact hashes.)
