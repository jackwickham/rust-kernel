use crate::peripherals::uart0::get_uart;
use crate::peripherals::power;

use core::fmt;
use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let mut uart = get_uart();
    uart.init();
    
    fmt::write(&mut uart, format_args!("{:?}", info));

    power::get_power_manager().reboot()
}
