mod ns16550a;

use crate::config::VIRT_UART;
use alloc::sync::Arc;
use lazy_static::lazy_static;

pub trait CharDevice {
    fn init(&self);
    fn read(&self) -> u8;
    fn write(&self, ch: u8);
    fn handle_irq(&self);
}

type CharDeviceImpl = ns16550a::NS1650a<VIRT_UART>;

lazy_static! {
    pub static ref UART: Arc<CharDeviceImpl> = Arc::new(CharDeviceImpl::new());
}
