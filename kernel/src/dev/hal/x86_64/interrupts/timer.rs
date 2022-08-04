use crate::*;
use dev::hal::cpu;
use x86_64::structures::idt;

pub extern "x86-interrupt" fn timer_handler(_stack_frame: idt::InterruptStackFrame) {
    cpu::pic_end_of_interrupt(super::HardwareInterrupt::Timer);
}