use crate::{*, dev::{Write}, task::scheduler};
use dev::hal::pic;
use core::arch::asm;
use x86_64::{structures::idt, instructions};
use dev::hal::{interrupts, task::{self, TaskContext}};

pub extern "x86-interrupt" fn timer_handler(_stack_frame: idt::InterruptStackFrame) {
    pic::end_of_interrupt(dev::hal::interrupts::HardwareInterrupt::Timer);
}
