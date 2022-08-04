use crate::*;
use panic;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn invalid_tss_handler(stack_frame: idt::InterruptStackFrame, error_code: u64) {
    panic!("\n{:?}\nSegment selector index: {:#010x}\nStack frame: {:#?})", Exception::EXCEPTION_INVALID_TSS, error_code, stack_frame);
}