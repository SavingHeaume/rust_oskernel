# 首先通过 __alltraps 将 Trap 上下文保存在内核栈上，
# 然后跳转到使用 Rust 编写的 trap_handler 函数完成 Trap 分发及处理。
# 当 trap_handler 返回之后，使用 __restore 从保存在内核栈上的 Trap 上下文恢复寄存器。
# 最后通过一条 sret 指令回到应用程序执行。

.altmacro
# 将通用寄存器 x\n 的值保存到内核栈的偏移位置 n*8 处。
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm

    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    .align 2

__alltraps:
    # 交换 sp栈指针和 sscratch 的值
    csrrw sp, sscratch, sp
    sd x1, 1*8(sp)
    sd x3, 3*8(sp)
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr

    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    csrr t2, sscratch
    sd t2, 2*8(sp)

    # 加载kernel_satp
    ld t0, 34*8(sp)
    # 加载trap_handler
    ld t1, 36*8(sp)
    # 移动到kernel_sp
    ld sp, 35*8(sp)
    csrw satp, t0
    jr t1

__restore:
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0

    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1

    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    ld sp, 2*8(sp)
    sret
