#!/bin/sh
set -ev

RUSTFLAGS="-C link-arg=-Tstar.ld -C linker=rust-lld" \
  cargo build --bin star --target riscv64gc-unknown-none-elf

cp target/riscv64gc-unknown-none-elf/debug/star star.elf

qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -bios default \
    -smp 1 \
    -m 128M \
    -nographic \
    -d cpu_reset,unimp,guest_errors,int -D qemu.log \
    -serial mon:stdio \
    --no-reboot \
    -kernel star.elf