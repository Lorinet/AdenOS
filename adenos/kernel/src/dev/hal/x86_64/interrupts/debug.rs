use crate::*;
use panic;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn debug_handler(stack_frame: idt::InterruptStackFrame) {
    panic!("\n{:?}\nStack frame: {:#?})", Exception::EXCEPTION_DEBUG, stack_frame);
}