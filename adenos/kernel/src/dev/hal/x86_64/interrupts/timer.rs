use crate::*;
use dev::hal::pic;
use x86_64::structures::idt;

pub extern "x86-interrupt" fn timer_handler(_stack_frame: idt::InterruptStackFrame) {
    pic::end_of_interrupt(dev::hal::interrupts::HardwareInterrupt::Timer);
}
