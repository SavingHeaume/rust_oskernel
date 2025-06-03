# Makefile

# 定义常用路径变量
USER_DIR := ../user
FS_FUSE_DIR := ../fs_fuse
KERNEL_DIR := ../kernel
USER_TARGET_PATH := $(USER_DIR)/target/riscv64gc-unknown-none-elf/release

.PHONY: all build_user build_fs build_kernel run_qemu clean

all: build_user build_fs build_kernel run_qemu

build_user:
	@echo "Building user..."
	cd $(USER_DIR) && \
	cargo clean && \
	cargo build --release

build_fs: build_user
	@echo "Building fs..."
	cd $(FS_FUSE_DIR) && \
	rm -f $(USER_TARGET_PATH)/fs.img && \
	cargo clean && \
	cargo run --release -- -s $(USER_DIR)/src/bin/ -t $(USER_TARGET_PATH)/

build_kernel:
	@echo "Building kernel..."
	cd $(KERNEL_DIR) && \
	cargo clean && \
	cargo build --release && \
	~/.cargo/bin/rust-objcopy \
		--strip-all \
		target/riscv64gc-unknown-none-elf/release/kernel \
		-O binary \
		target/riscv64gc-unknown-none-elf/release/kernel.bin

run_qemu:
	@echo "Starting QEMU..."
	cd $(KERNEL_DIR) && \
	qemu-system-riscv64 \
		-machine virt \
		-bios ../bootloader/rustsbi-qemu.bin \
		-serial stdio \
		-device loader,file=target/riscv64gc-unknown-none-elf/release/kernel.bin,addr=0x80200000 \
		-drive file=$(USER_TARGET_PATH)/fs.img,if=none,format=raw,id=x0 \
		-device virtio-blk-device,drive=x0 \
		-device virtio-gpu-device \
		-device virtio-keyboard-device \
		-device virtio-mouse-device \

clean:
	cd $(USER_DIR) && cargo clean
	cd $(FS_FUSE_DIR) && cargo clean
	cd $(KERNEL_DIR) && cargo clean
	rm -f $(USER_TARGET_PATH)/fs.img
	rm -f $(KERNEL_DIR)/target/riscv64gc-unknown-none-elf/release/kernel.bin
