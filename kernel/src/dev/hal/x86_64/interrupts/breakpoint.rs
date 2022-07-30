use crate::*;
use x86_64;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn breakpoint_handler(stack_frame: idt::InterruptStackFrame) {
    println!("{:?}\nStack frame: {:#?}", Exception::EXCEPTION_BREAKPOINT, stack_frame);
}

pub fn trigger_breakpoint() {
    x86_64::instructions::interrupts::int3();
}