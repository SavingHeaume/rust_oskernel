import os


def generate_link_app_s(
    user_bin_dir: str = "../user/src/bin",
    target_path: str = "../user/target/riscv64gc-unknown-none-elf/release/",
    output_file: str = "src/link_app.S"
) -> None:
    """
    生成汇编文件 link_app.S，嵌入用户程序的二进制数据，并包含应用名称
    """
    # 1. 获取所有用户程序名（去掉扩展名）
    apps = []
    for filename in os.listdir(user_bin_dir):
        if os.path.isfile(os.path.join(user_bin_dir, filename)):
            name, ext = os.path.splitext(filename)
            apps.append(name)
    apps = sorted(apps)  # 按名称排序

    # 2. 生成汇编内容
    asm_content = []
    
    # 2.1 写入应用数量 _num_app
    asm_content.append(
        f""".align 3
.section .data
.global _num_app
_num_app:
    .quad {len(apps)}"""
    )

    # 2.2 写入起始地址数组
    for i in range(len(apps)):
        asm_content.append(f"    .quad app_{i}_start")
    
    # 2.3 写入最后一个应用的结束地址
    if apps:
        asm_content.append(f"    .quad app_{len(apps)-1}_end")

    # 2.4 新增：写入应用名称字符串 _app_names
    asm_content.append(
        """
    .global _app_names
_app_names:"""
    )
    for app in apps:
        asm_content.append(f'    .string "{app}"')

    # 2.5 嵌入每个应用的二进制数据
    for idx, app in enumerate(apps):
        # 构建二进制文件路径（确保 target_path 以 / 结尾）
        bin_path = os.path.join(target_path, app)
        if not target_path.endswith("/"):
            bin_path = f"{target_path}/{app}"

        asm_content.append(
            f"""
    .section .data
    .global app_{idx}_start
    .global app_{idx}_end
    .align 3
app_{idx}_start:
    .incbin "{bin_path}"
app_{idx}_end:"""
        )

    # 3. 写入文件
    os.makedirs(os.path.dirname(output_file), exist_ok=True)
    with open(output_file, "w") as f:
        f.write("\n".join(asm_content))


def main():
    # generate_link_app_s()
    # os.system("cargo build --release")
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


if __name__ == "__main__":
    main()