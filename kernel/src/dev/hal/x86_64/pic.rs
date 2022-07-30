use crate::*;
use spin;
use dev::hal::cpu;
use pic8259::ChainedPics;
use x86_64::instructions;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn init() {
    unsafe { PICS.lock().initialize() };
}

pub fn end_of_interrupt(int: cpu::HardwareInterrupt) {
    disable_interrupts();
    unsafe {
        PICS.lock().notify_end_of_interrupt(int.as_u8());
    }
    enable_interrupts();
}

pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

pub fn disable_interrupts() {
    instructions::interrupts::disable();
}
