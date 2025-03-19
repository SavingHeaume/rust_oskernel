import os

os.system("cargo build --release")
os.system(
  "~/.cargo/bin/rust-objcopy "
  "--strip-all "
  "target/riscv64gc-unknown-none-elf/release/kernel "
  "-O binary "
  "target/riscv64gc-unknown-none-elf/release/kernel.bin"
)

os.system("qemu-system-riscv64 "
  "-machine virt "
  "-nographic "
  "-bios ../bootloader/rustsbi-qemu.bin "
  "-device loader,file=target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000"
)