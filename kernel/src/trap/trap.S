# 首先通过 __alltraps 将 Trap 上下文保存在内核栈上，
# 然后跳转到使用 Rust 编写的 trap_handler 函数完成 Trap 分发及处理。
# 当 trap_handler 返回之后，使用 __restore 从保存在内核栈上的 Trap 上下文恢复寄存器。
# 最后通过一条 sret 指令回到应用程序执行。

.altmacro
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
    csrrw sp, sscratch, sp
    # 现在 sp->*TrapContext 在用户空间，sscratch->用户堆栈
    # 保存其他通用寄存器
    sd x1, 1*8(sp)
    # skip sp(x2)
    sd x3, 3*8(sp)
    # skip tp(x4)
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
   
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # 从sscratch读取用户堆栈并将其保存在trap上下文中
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 加载kernel_satp到 t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp
    ld sp, 35*8(sp)
    # switch to kernel space
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *用户空间中的TrapContext（常量）；a1: 用户空间token
    # 切换到用户空间
    csrw satp, a1
    sfence.vma
    csrw sscratch, a0
    mv sp, a0
    # 现在 sp 指向用户空间中的 TrapContext，并以此为基础开始恢复
    # 恢复 sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # 恢复除 x0/sp/tp 之外的通用寄存器
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # 返回用户栈
    ld sp, 2*8(sp)
    sret
