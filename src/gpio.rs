/*
 * MIT License
 *
 * Copyright (c) 2018 Andre Richter <andre.o.richter@gmail.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

use super::MMIO_BASE;
use register::{mmio::ReadWrite, register_bitfields, IntLike, RegisterLongName};
use core::ops::Deref;

// Descriptions taken from
// https://github.com/raspberrypi/documentation/files/1888662/BCM2837-ARM-Peripherals.-.Revised.-.V2-1.pdf
register_bitfields! {
    u32,

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            RXD0 = 0b100, // UART0     - Alternate function 0
            RXD1 = 0b010  // Mini UART - Alternate function 5

        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            TXD0 = 0b100, // UART0     - Alternate function 0
            TXD1 = 0b010  // Mini UART - Alternate function 5
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

pub struct Reg<T: IntLike, R: RegisterLongName = ()> {
    r: *mut ReadWrite<T, R>
}

impl<T: IntLike, R: RegisterLongName> Reg<T, R> {
    pub const unsafe fn new(r: *mut T) -> Self {
        Reg {
            r: r as *mut ReadWrite<T, R>
        }
    }
}

impl<T: IntLike, R: RegisterLongName> Deref for Reg<T, R> {
    type Target = ReadWrite<T, R>;

    fn deref(&self) -> &ReadWrite<T, R> {
        unsafe {
            &*self.r
        }
    }
}

pub const GPFSEL1: Reg<u32, GPFSEL1::Register> = {
    unsafe {
        Reg::new((MMIO_BASE + 0x0020_0004) as *mut u32)
    }
};

pub const GPPUD: Reg<u32> = {
    unsafe {
        Reg::new((MMIO_BASE + 0x0020_0094) as *mut u32)
    }
};

pub const GPPUDCLK0: Reg<u32, GPPUDCLK0::Register> = {
    unsafe {
        Reg::new((MMIO_BASE + 0x0020_0098) as *mut u32)
    }
};