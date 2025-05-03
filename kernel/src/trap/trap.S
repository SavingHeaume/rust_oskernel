.altmacro
.macro SAVE_GP n          # 宏定义：将寄存器 x\n 存储到栈偏移 \n*8 的位置
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n          # 宏定义：从栈偏移 \n*8 的位置加载数据到 x\n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline  # 定义代码段 .text.trampoline
    .globl __alltraps         # 导出用户态 Trap 入口符号
    .globl __restore          # 导出用户态 Trap 恢复符号
    .globl __alltraps_k       # 导出内核态 Trap 入口符号
    .globl __restore_k        # 导出内核态 Trap 恢复符号
    .align 2                  # 按 4 字节对齐代码

# ==================== 用户态 Trap 处理 ====================
__alltraps:
    csrrw sp, sscratch, sp    # 交换 sp 和 sscratch，此时 sp 指向用户 Trap 上下文
    sd x1, 1*8(sp)           # 保存返回地址 (x1/ra)
    sd x3, 3*8(sp)           # 保存全局指针 (x3/gp)，跳过 x2(sp)/x4(tp)
    .set n, 5                # 从 x5 开始循环保存通用寄存器
    .rept 27                 # 保存 x5~x31（共27个寄存器）
        SAVE_GP %n
        .set n, n+1
    .endr
    # 保存关键控制寄存器
    csrr t0, sstatus         # 读取当前状态寄存器 sstatus
    csrr t1, sepc            # 读取异常程序计数器 sepc
    sd t0, 32*8(sp)          # 存储 sstatus 到 Trap 上下文第32项
    sd t1, 33*8(sp)          # 存储 sepc 到第33项
    csrr t2, sscratch        # 从 sscratch 获取用户栈指针
    sd t2, 2*8(sp)           # 保存用户栈指针到 Trap 上下文的 x2 位置
    # 切换到内核环境
    ld t0, 34*8(sp)          # 加载内核页表寄存器 satp 的值
    ld t1, 36*8(sp)          # 加载用户态 Trap 处理函数地址
    ld sp, 35*8(sp)          # 切换 sp 到内核栈
    csrw satp, t0            # 设置 satp 切换到内核地址空间
    sfence.vma               # 刷新 TLB 确保地址空间切换生效
    jr t1                    # 跳转到用户态 Trap 处理函数

__restore:
    csrw satp, a1            # 恢复用户页表（a1 为用户空间 satp 值）
    sfence.vma               # 刷新 TLB
    csrw sscratch, a0        # 将用户 Trap 上下文地址存入 sscratch
    mv sp, a0                # sp 指向用户 Trap 上下文
    # 恢复控制寄存器
    ld t0, 32*8(sp)          # 加载保存的 sstatus
    ld t1, 33*8(sp)          # 加载保存的 sepc
    csrw sstatus, t0         # 恢复 sstatus
    csrw sepc, t1            # 恢复 sepc
    # 恢复通用寄存器
    ld x1, 1*8(sp)           # 恢复返回地址 (x1/ra)
    ld x3, 3*8(sp)           # 恢复全局指针 (x3/gp)
    .set n, 5
    .rept 27                 # 恢复 x5~x31
        LOAD_GP %n
        .set n, n+1
    .endr
    ld sp, 2*8(sp)           # 恢复用户栈指针（从 Trap 上下文的 x2 位置）
    sret                     # 返回用户态执行

# ==================== 内核态 Trap 处理 ====================
    .align 2
__alltraps_k:
    addi sp, sp, -34*8       # 在内核栈分配 34 项空间（x1~x31 + sstatus + sepc）
    sd x1, 1*8(sp)           # 保存返回地址 (x1/ra)
    sd x3, 3*8(sp)           # 保存全局指针 (x3/gp)
    .set n, 5
    .rept 27                 # 保存 x5~x31
        SAVE_GP %n
        .set n, n+1
    .endr
    # 保存内核态控制寄存器
    csrr t0, sstatus         # 读取当前 sstatus
    csrr t1, sepc            # 读取当前 sepc
    sd t0, 32*8(sp)          # 存储 sstatus 到栈第32项
    sd t1, 33*8(sp)          # 存储 sepc 到栈第33项
    mv a0, sp                # 将栈指针作为参数传递给处理函数（a0）
    csrr t2, sscratch        # 从 sscratch 读取内核处理函数地址
    jalr t2                  # 跳转到处理函数，同时保存返回地址到 ra

__restore_k:
    # 恢复内核态控制寄存器
    ld t0, 32*8(sp)          # 加载保存的 sstatus
    ld t1, 33*8(sp)          # 加载保存的 sepc
    csrw sstatus, t0         # 恢复 sstatus
    csrw sepc, t1            # 恢复 sepc
    # 恢复通用寄存器
    ld x1, 1*8(sp)           # 恢复返回地址 (x1/ra)
    ld x3, 3*8(sp)           # 恢复全局指针 (x3/gp)
    .set n, 5
    .rept 27                 # 恢复 x5~x31
        LOAD_GP %n
        .set n, n+1
    .endr
    addi sp, sp, 34*8        # 释放内核栈空间（34 项 * 8 字节）
    sret                     # 返回内核态继续执行
