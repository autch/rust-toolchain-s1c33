#!/bin/sh
# Build and run the s1c33 ABI unit tests under piece-emu (semihosting).
# Exit 0 = all pass; non-zero = a check failed (the emulator prints the code).
#
# Paths are repo-relative; llvm-c33 is reached via the in-tree symlink.
set -e
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/.." && pwd)
rust="$root/rust/build/x86_64-unknown-linux-gnu"
llvm="$root/llvm-c33"                                   # in-tree symlink
bm="$llvm/piece-emu/src/tests/bare_metal"
sysroot="$llvm/sysroot/s1c33-none-piece"
out="$here/build"
mkdir -p "$out"
cd "$here"

# 1. Rust test staticlib (build-std=core).
RUSTC="$rust/stage1/bin/rustc" RUSTC_BOOTSTRAP=1 \
  "$rust/stage0/bin/cargo" build --release

# 2. C fixtures + bare-metal crt, built from source with clang (s1c33-none-elf).
cc() { "$llvm/build/bin/clang" --target=s1c33-none-elf --sysroot="$sysroot" "$@"; }
cc -O1 -ffreestanding -c csrc/abi_test.c   -o "$out/abi_test.o"
cc -c "$bm/crt0.s"                         -o "$out/crt0.o"
cc -O1 -ffreestanding -c "$bm/crt_init.c"  -o "$out/crt_init.o"

# 3. Link bare-metal into IRAM (iram.ld, entry _start).
"$llvm/build/bin/ld.lld" -T "$bm/iram.ld" --entry _start -o "$out/abitest.elf" \
  "$out/crt0.o" "$out/crt_init.o" "$out/abi_test.o" \
  target/s1c33-none-piece/release/libabitest.a \
  "$sysroot/lib/libclang_rt.builtins-s1c33.a"

# 4. Run under the headless emulator (exit code == test result).
exec "$llvm/piece-emu/build-src/piece-emu" --max-cycles 2000000 "$out/abitest.elf"
