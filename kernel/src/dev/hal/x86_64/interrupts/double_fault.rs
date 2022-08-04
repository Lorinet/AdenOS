use crate::*;
use panic;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn double_fault_handler(stack_frame: idt::InterruptStackFrame, error_code: u64) -> ! {
    panic!("\n{:?}\nError code: {:#010x}\nStack frame: {:#?})", Exception::EXCEPTION_DOUBLE_FAULT, error_code, stack_frame);
}