use crate::*;
use dev::hal::cpu;
use core::arch::asm;
use x86_64::{structures::idt, instructions};

pub extern "x86-interrupt" fn timer_handler(_stack_frame: idt::InterruptStackFrame) {
    cpu::pic_end_of_interrupt(super::HardwareInterrupt::Timer);
}