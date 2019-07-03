#
# MIT License
#
# Copyright (c) 2018-2019 Andre Richter <andre.o.richter@gmail.com>
#
# Permission is hereby granted, free of charge, to any person obtaining a copy
# of this software and associated documentation files (the "Software"), to deal
# in the Software without restriction, including without limitation the rights
# to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
# copies of the Software, and to permit persons to whom the Software is
# furnished to do so, subject to the following conditions:
#
# The above copyright notice and this permission notice shall be included in all
# copies or substantial portions of the Software.
#
# THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
# IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
# FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
# AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
# LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
# OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
# SOFTWARE.
#

TARGET = aarch64-unknown-none

SOURCES = $(wildcard **/*.rs) $(wildcard **/*.S) link.ld

CARGO_OUTPUT = target/$(TARGET)/release/kernel

BUILD_VERSION = --release

.PHONY: all qemu clippy clean objdump nm

all: clean kernel8.img

$(CARGO_OUTPUT): $(SOURCES)
	cargo xrustc $(BUILD_VERSION)

kernel8.img: $(CARGO_OUTPUT)
	cp $< ./kernel8
	cargo objcopy -- --strip-all -O binary $< kernel8.img

clippy:
	cargo xclippy --target=$(TARGET)

clean:
	cargo clean

objdump:
	cargo objdump -- -disassemble -print-imm-hex kernel8

nm:
	cargo nm -- kernel8 | sort