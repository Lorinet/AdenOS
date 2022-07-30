use crate::*;
use panic;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: idt::InterruptStackFrame, error_code: u64) -> ! {
    panic::trigger_panic_exception(&Exception::EXCEPTION_DOUBLE_FAULT, &stack_frame, error_code);
}