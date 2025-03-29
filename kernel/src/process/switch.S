.altmacro 
.macro SAVE_SN n
  sd s\n, (\n+2)*8(a0)
.endm

.macro LOAD_SN n
  ld s\n, (\n+2)*8(a1)
.endm

  .section .text
  .globl __switch

__switch:
  # 保存当前任务上下文
  sd sp, 8(a0)    # 保存当前内核栈指针 sp 到 TaskContext.sp 字段（偏移 8）
  sd ra, 0(a0)    # 保存返回地址 ra 到 TaskContext.ra 字段（偏移 0）

  .set n, 0       # 初始化计数器 n=0（对应 s0 寄存器）
  .rept 12        # 重复 12 次（保存 s0~s11）调用宏 SAVE_SN，保存 s[n] 到偏移 (n+2)*8 处
    SAVE_SN %n  
    .set n, n+1 
  .endr

  # 加载下一个任务上下文
  ld ra, 0(a1)    # 从下一个任务的 TaskContext.ra 加载返回地址

  .set n, 0       # 初始化计数器 n=0
  .rept 12        # 重复 12 次（加载 s0~s11）
      LOAD_SN %n  # 调用宏 LOAD_SN，从偏移 (n+2)*8 处加载到 s[n]
      .set n, n+1
  .endr

  ld sp, 8(a1)    # 加载下一个任务的内核栈指针 sp

  # 切换控制流
  ret  # 跳转到 ra 寄存器指向的地址（即下一个任务的执行点）