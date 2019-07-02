use super::{MMIO_BASE};
use register::{mmio::{ReadOnly, WriteOnly}, register_bitfields};
use core::hint::spin_loop;
use core::sync::atomic::{compiler_fence, Ordering, AtomicU8, AtomicU32};
use core::slice;

register_bitfields!{
    u32,
    STATUS [
        FULL    OFFSET(31) NUMBITS(1) [],
        EMPTY   OFFSET(30) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct Mailbox0Registers {
    DATA: ReadOnly<u32>,                        // 0x00
    __reserved_0: [u32; 3],                     // 0x04
    PEEK: ReadOnly<u32>,                        // 0x10
    SENDER: ReadOnly<u32>,                      // 0x14
    STATUS: ReadOnly<u32, STATUS::Register>,    // 0x18
    CONFIG: ReadOnly<u32>,                      // 0x1C
}


#[allow(non_snake_case)]
#[repr(C)]
pub struct Mailbox1Registers {
    DATA: WriteOnly<u32>,                       // 0x20
    __reserved_0: [u32; 3],                     // 0x24
    PEEK: ReadOnly<u32>,                        // 0x30
    SENDER: ReadOnly<u32>,                      // 0x34
    STATUS: ReadOnly<u32, STATUS::Register>,    // 0x38
    CONFIG: ReadOnly<u32>,                      // 0x3C
}

fn get_mailbox_0() -> &'static Mailbox0Registers {
    unsafe {
        &*((MMIO_BASE + 0xB880) as *const Mailbox0Registers)
    }
}
fn get_mailbox_1() -> &'static Mailbox1Registers {
    unsafe {
        &*((MMIO_BASE + 0xB880 + 0x20) as *const Mailbox1Registers)
    }
}

#[derive(Debug)]
pub enum MailboxError {
    ResponseError,
    SizeError(u32),
    OverflowError,
    UnknownError,
}
pub type Result<T> = ::core::result::Result<T, MailboxError>;

#[derive(Copy, Clone)]
#[allow(dead_code)]
#[repr(u32)]
pub enum Channel {
    PowerManagement = 0,
    FrameBuffer = 1,
    VirtualUART = 2,
    VCHIQ = 3,
    LEDs = 4,
    Buttons = 5,
    TouchScreen = 6,
    PropertyTagsVC = 8,
    PropertyTagsCPU = 9
}

#[allow(dead_code)]
#[repr(u32)]
pub enum Tag {
    Last = 0,
    GetFirmware = 0x0000_0001,
    GetModel = 0x0001_0001,
    GetRevision = 0x0001_0002,
    GetMac = 0x0001_0003,
    GetSerial = 0x0001_0004,
    GetArmMemory = 0x0001_0005,
    GetVCMemory = 0x0001_0006,
    GetClocks = 0x0001_0007,

    SetClockRate = 0x0003_8002,
}

#[derive(PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Clock {
    UART = 0x0000_0002,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[repr(u32)]
pub enum RequestResponseCode {
    REQUEST = 0x0,

    SUCCESS = 0x8000_0000,
    ERROR = 0x8000_0001,
}

const MESSAGE_SIZE: usize = 12;

/// Buffer for the messages to exchange with the GPU
#[repr(C)]
#[repr(align(16))]
pub struct Message {
    pub message_size: AtomicU32,
    pub request_response_code: AtomicU32,
    pub tag_identifier: AtomicU32,
    pub value_buffer_size: AtomicU32,
    pub value_length: AtomicU32,
    pub buffer: [AtomicU32; MESSAGE_SIZE],
    reserved: u32,
}

impl Message {
    /// Construct a new message to send
    pub fn new() -> Message {
        let buf: [AtomicU32; MESSAGE_SIZE] = Default::default();
        Message {
            message_size: AtomicU32::new((MESSAGE_SIZE + 6) as u32 * 4),
            request_response_code: AtomicU32::new(RequestResponseCode::REQUEST as u32),
            tag_identifier: AtomicU32::new(Tag::Last as u32),
            value_buffer_size: AtomicU32::new(MESSAGE_SIZE as u32 * 4),
            value_length: AtomicU32::new(0),
            buffer: buf,
            reserved: 0,
        }
    }

    pub fn call(self, channel: Channel) -> Result<Message> {
        // The address of the buffer has to be coerced into a 32 bit pointer.
        // Because we are using physical addresses and there's max 1GB of RAM,
        // the address will fit within that (even though it's a 64 bit CPU)
        let buf_ptr = &self as *const Message as u32;
        
        // The message is the upper 28 bits of the pointer in the upper 28 bits
        // of the message, and the channel in the lower 4 bits
        let msg: u32 = (buf_ptr & !0x0F) | (channel as u32);

        // Wait for there to be space in the mailbox (I think that should always
        // be the case anyway)
        while get_mailbox_1().STATUS.is_set(STATUS::FULL) {
            spin_loop();
        }

        // Make sure no memory operations cross this point.
        // The processor on the pi executes in-order, so it doesn't need to be
        // exposed as a fence in the actual instructions
        compiler_fence(Ordering::Release);

        // Send it!
        get_mailbox_1().DATA.set(msg);

        // Wait for the response
        let mailbox0 = get_mailbox_0();
        loop {
            if !mailbox0.STATUS.is_set(STATUS::EMPTY) {
                // Peek at the message and see if it's for us
                let resp: u32 = mailbox0.DATA.get();
                if (resp & !0x0F) == (buf_ptr & !0x0F) {
                    // It is for us, so pop it off then check whether it's the
                    // response that we care about
                    if (resp & 0x0F) == (channel as u32) {
                        // This is the one we're interested in
                        // First we need to insert a barrier so we can't speculate this
                        compiler_fence(Ordering::Acquire);

                        return match self.request_response_code.load(Ordering::Relaxed) {
                            v if v == RequestResponseCode::SUCCESS as u32 => Ok(self),
                            v if v == RequestResponseCode::ERROR as u32 => Err(MailboxError::ResponseError),
                            v => Err(MailboxError::UnknownError),
                        }
                    }
                }
            }

            spin_loop();
        }
    }

    pub fn send_property(self, tag: Tag, expected_len: u32) -> Result<Message> {
        if expected_len > MESSAGE_SIZE as u32 {
            return Err(MailboxError::OverflowError);
        }
        self.tag_identifier.store(tag as u32, Ordering::Relaxed);

        // send it
        let result = self.call(Channel::PropertyTagsVC)?;

        if result.value_length.load(Ordering::Relaxed) & 0x8000_0000 != 0x8000_0000 {
            return Err(MailboxError::UnknownError);
        }

        let size = result.value_length.load(Ordering::Relaxed) & 0x7FFF_FFFF;
        if size != expected_len {
            Err(MailboxError::SizeError(size))
        } else {
            Ok(result)
        }
    }

    /// Get the firmware revision of this board
    pub fn get_firmware_revision() -> Result<u32> {
        let message = Message::new();
        let result = message.send_property(Tag::GetFirmware, 4)?;
        Ok(result.buffer[0].load(Ordering::Relaxed))
    }

    /// Get the mac address of this board
    pub fn get_mac() -> Result<[u8; 6]> {
        let message = Message::new();
        let result = message.send_property(Tag::GetMac, 6)?;
        let mac: &[AtomicU8] = unsafe {
            slice::from_raw_parts(result.buffer.as_ptr() as *const AtomicU8, 6)
        };
        Ok([
            mac[0].load(Ordering::Relaxed), mac[1].load(Ordering::Relaxed),
            mac[2].load(Ordering::Relaxed), mac[3].load(Ordering::Relaxed),
            mac[4].load(Ordering::Relaxed), mac[5].load(Ordering::Relaxed)
        ])
    }

    /// Get the serial number of this board
    pub fn get_serial() -> Result<u64> {
        let message = Message::new();
        let result = message.send_property(Tag::GetSerial, 8)?;
        Ok(
            result.buffer[0].load(Ordering::Relaxed) as u64 +
            ((result.buffer[1].load(Ordering::Relaxed) as u64) << 32)
        )
    }

    /// Get the base address and size of the ram allocated to the ARM core
    pub fn get_memory_range() -> Result<(u32, u32)> {
        let message = Message::new();
        let result = message.send_property(Tag::GetArmMemory, 8)?;
        Ok((result.buffer[0].load(Ordering::Relaxed), result.buffer[1].load(Ordering::Relaxed)))
    }

    pub fn set_clock_rate(clock: Clock, rate: u32, skip_setting_turbo: u32) -> Result<u32> {
        let message = Message::new();
        message.buffer[0].store(clock as u32, Ordering::Relaxed);
        message.buffer[1].store(rate, Ordering::Relaxed);
        message.buffer[2].store(skip_setting_turbo, Ordering::Relaxed);

        let result = message.send_property(Tag::SetClockRate, 8)?;
        Ok(result.buffer[1].load(Ordering::Relaxed))
    }

    fn flush_cache_line(&self) {
        let addr = self as *const Message as usize;
        unsafe {
            asm!("DC CIVAC, $0" :: "r"(addr) :: "volatile");
        }
        compiler_fence(Ordering::SeqCst);
    }
}