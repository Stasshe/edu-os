use pic8259::ChainedPics;
use spin;
use lazy_static::lazy_static;
use pc_keyboard::{layouts, HandleControl, PS2Keyboard,ScancodeSet1};
use spin::Mutex;


pub const PIC_1_OFFSET: u8 = 32;
// cpu例外0-31避けてここから
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET)});

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub fn as_u8(self) -> u8 {
        self as u8
    }
    pub fn as_usize(self) -> usize {
        self.as_u8() as usize
    }
}


lazy_static! {
    pub static ref KEYBOARD: Mutex<PS2Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(PS2Keyboard::new(
            ScancodeSet1::new(),
            layouts::Us104Key,
            HandleControl::Ignore,
    ));
}
