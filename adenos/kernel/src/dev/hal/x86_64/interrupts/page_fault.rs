use crate::*;
use panic;
use x86_64::structures::idt;
use x86_64::registers::control::Cr2;
use super::*;

pub extern "x86-interrupt" fn page_fault_handler(stack_frame: idt::InterruptStackFrame, error_code: idt::PageFaultErrorCode) {
    panic!("\n{:?}\nError code: {:?}\nAccessed address: {:?}\nStack frame: {:#?})", Exception::EXCEPTION_PAGE_FAULT, error_code, Cr2::read(), stack_frame);
}