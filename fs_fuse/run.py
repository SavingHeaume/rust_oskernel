import os

os.system("rm -f ../user/target/riscv64gc-unknown-none-elf/release/fs.img")
os.system("cargo clean")
os.system("cargo run --release -- -s ../user/src/bin/ -t ../user/target/riscv64gc-unknown-none-elf/release/")
