mod context;

use crate::config::{TRAMPOLINE, TRAP_CONTEXT};
use crate::syscall::syscall;
use crate::task::{
    SignalFlags, check_signals_error_of_current, current_add_signal, current_trap_cx,
    current_user_token, exit_current_and_run_next, handle_signals, suspend_current_and_run_next,
};
use crate::timer::set_next_trigger;
use core::arch::{asm, global_asm};
use log::*;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Interrupt, Trap},
    sie, stval, stvec,
};

global_asm!(include_str!("trap.S"));

pub fn init() {
    set_kernel_trap_entry();
}

fn set_kernel_trap_entry() {
    unsafe {
        stvec::write(trap_from_kernel as usize, TrapMode::Direct);
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
            suspend_current_and_run_next();
        }
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    handle_signals();

    if let Some((errno, msg)) = check_signals_error_of_current() {
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
    set_user_trap_entry();
    let trap_cx_ptr = TRAP_CONTEXT;
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
            in("a0") trap_cx_ptr,
            in("a1") user_satp,
            options(noreturn)
        );
    }
}

#[unsafe(no_mangle)]
pub fn trap_from_kernel() -> ! {
    use riscv::register::sepc;
    error!(
        "[trap] stval = {:#x}, sepc = {:#x}",
        stval::read(),
        sepc::read()
    );
    panic!("a trap {:?} from kernel!", scause::read().cause());
}

pub use context::TrapContext;
