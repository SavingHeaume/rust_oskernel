use riscv::register::sstatus::{self, SPP, Sstatus};

#[repr(C)]
#[derive(Debug)]
/// 包含所有的通用寄存器 x0~x31 ，还有 sstatus 和 sepc
pub struct TrapContext {
    /// 通用寄存器 x0-31
    pub x: [usize; 32],
    pub sstatus: Sstatus,
    pub sepc: usize,
    /// 内核地址空间的 token ，即内核页表的起始物理地址
    pub kernel_satp: usize,
    ///  当前应用在内核地址空间中的内核栈栈顶的虚拟地址；
    pub kernel_sp: usize,
    /// 内核中 trap handler 入口点的虚拟地址。
    pub trap_handler: usize,
}

impl TrapContext {
    /// 将 sp堆栈指针放入 TrapContext 的 x[2] 字段中
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }
    /// 初始化应用程序的trap上下文
    pub fn app_init_context(
        entry: usize,
        sp: usize,
        kernel_satp: usize,
        kernel_sp: usize,
        trap_handler: usize,
    ) -> Self {
        let mut sstatus = sstatus::read();
        // 将 CPU 权限设置为用户 after trapping back
        sstatus.set_spp(SPP::User);
        let mut cx = Self {
            x: [0; 32],
            sstatus,
            sepc: entry,  // 应用程序入口点
            kernel_satp,  // 页表地址
            kernel_sp,    
            trap_handler, 
        };
        cx.set_sp(sp); // 应用程序的用户栈指针
        cx 
    }
}
