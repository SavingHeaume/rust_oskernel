mod context;

use crate::config::TRAMPOLINE;
use crate::syscall::syscall;
use crate::task::{
    SignalFlags, check_signals_of_current, current_add_signal, current_trap_cx,
    current_trap_cx_user_va, current_user_token, exit_current_and_run_next,
    suspend_current_and_run_next,
};
use crate::timer::{check_timer, set_next_trigger};
use core::arch::{asm, global_asm};
use log::*;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, sscratch, sstatus, stval, stvec,
};

global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe extern "C" {
        unsafe fn __alltraps();
        unsafe fn __alltraps_k();
    }
    let __alltraps_k_va = __alltraps_k as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        stvec::write(__alltraps_k_va, TrapMode::Direct);
        sscratch::write(trap_from_kernel as usize);
    }
}

fn set_user_trap_entry() {
    unsafe {
        stvec::write(TRAMPOLINE as usize, TrapMode::Direct);
    }
}

/// 在supervisor模式下启用定时器中断
pub fn enable_timer_interrupt() {
    unsafe {
        sie::set_stimer();
    }
}

pub fn enable_supervisor_interrupt() {
    unsafe {
        sstatus::set_sie();
    }
}

pub fn disable_supervisor_interrupt() {
    unsafe {
        sstatus::clear_sie();
    }
}

#[unsafe(no_mangle)]
pub fn trap_handler() -> ! {
    set_kernel_trap_entry();
    let scause = scause::read();
    let stval = stval::read();
    match scause.cause() {
        // 应用程序发起系统调用
        Trap::Exception(Exception::UserEnvCall) => {
            // info!("trap due to system call");
            let mut cx = current_trap_cx();
            cx.sepc += 4;

            enable_supervisor_interrupt();

            let result = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]);
            cx = current_trap_cx();
            cx.x[10] = result as usize;
        }

        // 处理应用程序出现访存错误
        Trap::Exception(Exception::StoreFault)
        | Trap::Exception(Exception::StorePageFault)
        | Trap::Exception(Exception::InstructionFault)
        | Trap::Exception(Exception::InstructionPageFault)
        | Trap::Exception(Exception::LoadFault)
        | Trap::Exception(Exception::LoadPageFault) => {
            info!("[trap] trap due to page fault");
            current_add_signal(SignalFlags::SIGSEGV);
        }

        // 处理非法指令错误
        Trap::Exception(Exception::IllegalInstruction) => {
            info!("[trap] trap due to illegal instruction");
            current_add_signal(SignalFlags::SIGILL);
        }

        // 时间中断
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            // info!("trap due to time interrupt");
            set_next_trigger();
            check_timer();
            suspend_current_and_run_next();
        }

        // 内核中断
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::config::irq_handler();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }

    if let Some((errno, msg)) = check_signals_of_current() {
        info!("[trap] {}", msg);
        exit_current_and_run_next(errno);
    }
    trap_return();
}

#[unsafe(no_mangle)]
/// set the new addr of __restore asm function in TRAMPOLINE page,
/// set the reg a0 = trap_cx_ptr, reg a1 = phy addr of usr page table,
/// finally, jump to new addr of __restore asm function
pub fn trap_return() -> ! {
    disable_supervisor_interrupt();
    set_user_trap_entry();
    let trap_cx_user_va = current_trap_cx_user_va();
    let user_satp = current_user_token();
    unsafe extern "C" {
        unsafe fn __alltraps();
        unsafe fn __restore();
    }
    let restore_va = __restore as usize - __alltraps as usize + TRAMPOLINE;
    unsafe {
        asm!(
            "fence.i",
            "jr {restore_va}",
            restore_va = in(reg) restore_va,
            in("a0") trap_cx_user_va,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel(_trap_cx: &TrapContext) {
    let scause = scause::read();
    let stval = stval::read();

    match scause.cause() {
        Trap::Interrupt(Interrupt::SupervisorExternal) => {
            crate::config::irq_handler();
        }
        Trap::Interrupt(Interrupt::SupervisorTimer) => {
            set_next_trigger();
            check_timer();
            // do not schedule now
        }
        _ => {
            panic!(
                "Unsupported trap from kernel: {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
}

pub use context::TrapContext;
