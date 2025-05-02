use alloc::collections::vec_deque::VecDeque;
use bitflags::*;
use volatile::{ReadOnly, Volatile, WriteOnly};

use crate::{sync::{Condvar, UPIntrFreeCell}, task::schedule};

use super::CharDevice;

bitflags! {
    /// 中断使能寄存器
    pub struct IER: u8 {
        /// 当接收缓冲区有数据时触发中断 (位0)
        const RX_AVAILABLE = 1 << 0;
        /// 当发送保持寄存器为空时触发中断 (位1)
        const TX_EMPTY = 1 << 1;
    }

    /// 线路状态寄存器
    pub struct LSR: u8 {
        /// 接收缓冲区有数据可读 (位0)
        const DATA_AVAILABLE = 1 << 0;
        /// 发送保持寄存器为空（可写入新数据）(位5)
        const THR_EMPTY = 1 << 5;
    }

    /// 调制解调器控制寄存器
    pub struct MCR: u8 {
        /// 数据终端就绪 (DTR) 信号 (位0)
        const DATA_TERMINAL_READY = 1 << 0;
        /// 请求发送 (RTS) 信号 (位1)
        const REQUEST_TO_SEND = 1 << 1;
        /// 辅助输出控制位1 (位2)
        const AUX_OUTPUT1 = 1 << 2;
        /// 辅助输出控制位2 (位3)
        const AUX_OUTPUT2 = 1 << 3;
    }
}

#[repr(C)]
#[allow(dead_code)]
struct ReadWithoutDLAB {
    /// 接收缓冲寄存器（只读）
    pub rbr: ReadOnly<u8>,
    /// 中断使能寄存器（可读写）
    pub ier: Volatile<IER>,
    /// 中断标识寄存器（只读）
    pub iir: ReadOnly<u8>,
    /// 线路控制寄存器（可读写）
    pub lcr: Volatile<u8>,
    /// 调制解调器控制寄存器（可读写）
    pub mcr: Volatile<MCR>,
    /// 线路状态寄存器（只读）
    pub lsr: ReadOnly<LSR>,
    /// 忽略调制解调器状态寄存器 (MSR)
    _padding1: ReadOnly<u8>,
    /// 忽略暂存寄存器 (SCR)
    _padding2: ReadOnly<u8>,
}

#[repr(C)]
#[allow(dead_code)]
struct WriteWithoutDLAB {
    /// 发送保持寄存器（只写）
    pub thr: WriteOnly<u8>,
    /// 中断使能寄存器（可读写）
    pub ier: Volatile<IER>,
    /// 忽略 FIFO 控制寄存器 (FCR)
    _padding0: ReadOnly<u8>,
    /// 线路控制寄存器（可读写）
    pub lcr: Volatile<u8>,
    /// 调制解调器控制寄存器（可读写）
    pub mcr: Volatile<MCR>,
    /// 线路状态寄存器（只读）
    pub lsr: ReadOnly<LSR>,
    /// 忽略其他未使用的寄存器
    _padding1: ReadOnly<u16>,
}

pub struct NS1650aRaw {
    base_addr: usize,
}

impl NS1650aRaw {
    fn read_end(&mut self) -> &mut ReadWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut ReadWithoutDLAB) }
    }

    fn write_end(&mut self) -> &mut WriteWithoutDLAB {
        unsafe { &mut *(self.base_addr as *mut WriteWithoutDLAB) }
    }

    pub fn new(base_addr: usize) -> Self {
        Self { base_addr }
    }

    pub fn init(&mut self) {
        let read_end = self.read_end();

        let mut mcr = MCR::empty();
        mcr |= MCR::DATA_TERMINAL_READY;
        mcr |= MCR::REQUEST_TO_SEND;
        mcr |= MCR::AUX_OUTPUT2;
        read_end.mcr.write(mcr);

        let ier = IER::RX_AVAILABLE;
        read_end.ier.write(ier);
    }

    pub fn read(&mut self) -> Option<u8> {
        let read_end = self.read_end();
        let lsr = read_end.lsr.read();
        if lsr.contains(LSR::DATA_AVAILABLE) {
            Some(read_end.rbr.read())
        } else {
            None
        }
    }

    pub fn write(&mut self, ch: u8) {
        let write_end = self.write_end();
        loop {
            if write_end.lsr.read().contains(LSR::THR_EMPTY) {
                write_end.thr.write(ch);
                break;
            }
        }
    }
}

struct NS1650aInner {
    ns16550a: NS1650aRaw,
    read_buffer: VecDeque<u8>,
}

pub struct NS1650a<const BASE_ADDR: usize> {
    inner: UPIntrFreeCell<NS1650aInner>,
    condvar: Condvar,
}

impl<const BASE_ADDR: usize> NS1650a<BASE_ADDR> {
    pub fn new() -> Self {
        let inner = NS1650aInner {
            ns16550a: NS1650aRaw::new(BASE_ADDR),
            read_buffer: VecDeque::new(),
        };

        Self {
            inner: unsafe { UPIntrFreeCell::new(inner) },
            condvar: Condvar::new(),
        }
    }

    pub fn read_buffer_is_empty(&self) -> bool {
        self.inner
            .exclusive_session(|inner| inner.read_buffer.is_empty())
    }
}

impl<const BASE_ADDR: usize> CharDevice for NS1650a<BASE_ADDR> {
    fn init(&self) {
        let mut inner = self.inner.exclusive_access();
        inner.ns16550a.init();
        drop(inner);
    }

    fn read(&self) -> u8 {
        loop {
            let mut inner = self.inner.exclusive_access();
            if let Some(ch) = inner.read_buffer.pop_front() {
                return ch;
            } else {
                let task_cx_ptr = self.condvar.wait_no_sched();
                drop(inner);
                schedule(task_cx_ptr);
            }
        }
    }

    fn write(&self, ch: u8) {
        let mut inner = self.inner.exclusive_access();
        inner.ns16550a.write(ch);
    }

    fn handle_irq(&self) {
        let mut count = 0;
        self.inner.exclusive_session(|inner| {
            while let Some(ch) = inner.ns16550a.read() {
                count += 1;
                inner.read_buffer.push_back(ch);
            }
        });
        if count > 0 {
            self.condvar.signal();
        }
    }
}
