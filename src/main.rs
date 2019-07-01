#![no_std]
#![no_main]
#![feature(asm)]
#![feature(global_asm)]
#![feature(renamed_spin_loop)]
#![feature(const_fn)]
#![feature(const_raw_ptr_deref)]
#![feature(never_type)]

extern crate register;

mod gpio;
mod uart1;
mod uart0;
mod mailbox;
mod panic_handler;
mod self_update;

const MMIO_BASE: usize = 0x3F00_0000;

fn entry() -> ! {
    let uart = uart0::get_uart();

    uart.init().unwrap();
    uart.puts("\nInitialising...\n");
    uart.newline();

    uart.puts("Mac address: ");
    match mailbox::Message::get_mac() {
        Ok(mac) => {
            for i in 0..6 {
                uart.send_hex_u8(mac[i]);
                uart.send(':');
            }
        },
        Err(e) => ::core::fmt::write(uart, format_args!("{:?}", e)).unwrap(),
    };
    uart.newline();

    uart.puts("Serial number: ");
    match mailbox::Message::get_serial() {
        Ok(serial) => {
            uart.send_hex_u32((serial >> 32) as u32);
            uart.send_hex_u32((serial & 0xFFFF_FFFF) as u32);
        },
        Err(e) => ::core::fmt::write(uart, format_args!("{:?}", e)).unwrap(),
    }
    uart.newline();

    match mailbox::Message::get_memory_range() {
        Ok((base, length)) => {
            uart.puts("Memory size: 0x");
            uart.send_hex_u32(length);
            uart.puts(". Base: 0x");
            uart.send_hex_u32(base);
            uart.newline();
        }
        Err(e) => ::core::fmt::write(uart, format_args!("{:?}", e)).unwrap(),
    }

    loop {
        let c = uart.getc();
        if c == '^' {
            //uart.puts("\nPreparing to update code\n");
            if let Err(e) = self_update::self_update(&uart) {
                ::core::fmt::write(uart, format_args!("{:?}", e));
            }
        } else if c == '\n' {
            uart.newline();
        } else {
            uart.send(c);
        }
    }
}

init::entry!(entry);


#[inline]
pub fn sleep(cycles: u64) {
    for _ in 0..cycles {
        unsafe {
            asm!("nop" :::: "volatile");
        }
    }
}
