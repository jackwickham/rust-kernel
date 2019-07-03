use crate::uart1::MiniUart;
use crate::power;

use core::fmt;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = MiniUart::new();
    uart.init();
    
    fmt::write(&mut uart, format_args!("{:?}", info));

    power::get_power_manager().reboot()
}
