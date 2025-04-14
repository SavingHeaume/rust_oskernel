import os


def main():
    # generate_link_app_s()
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
              "-device loader,file=target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000 "
              "-drive file=../user/target/riscv64gc-unknown-none-elf/release/fs.img,if=none,format=raw,id=x0 " 
              "-device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 "
            )


if __name__ == "__main__":
    main()
