use crate::*;
use panic;
use x86_64::structures::idt;
use super::*;

pub extern "x86-interrupt" fn stack_segment_fault_handler(stack_frame: idt::InterruptStackFrame, error_code: u64) {
    panic!("\n{:?}\nSegment selector index: {:#010x}\nStack frame: {:#?})", Exception::EXCEPTION_STACK_SEGMENT_FAULT, error_code, stack_frame);
}