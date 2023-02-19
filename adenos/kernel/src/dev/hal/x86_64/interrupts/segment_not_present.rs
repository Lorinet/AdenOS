use crate::*;
use panic;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn segment_not_present_handler(stack_frame: idt::InterruptStackFrame, error_code: u64) {
    panic!("\n{:?}\nSegment selector index: {:#010x}\nStack frame: {:#?})", Exception::EXCEPTION_SEGMENT_NOT_PRESENT, error_code, stack_frame);
}