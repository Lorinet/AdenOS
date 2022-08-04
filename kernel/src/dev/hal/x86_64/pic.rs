use crate::*;
use spin;
use super::interrupts;
use pic8259::ChainedPics;
use dev::hal::cpu;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    println!("Initializing PIC8259...");
    unsafe { PICS.lock().initialize() };
    println!("Enabling hardware interrupts...");
    cpu::enable_interrupts();
}

pub fn end_of_interrupt(int: interrupts::HardwareInterrupt) {
    cpu::disable_interrupts();
    unsafe {
        PICS.lock().notify_end_of_interrupt(int.as_u8());
    }
    cpu::enable_interrupts();
}
