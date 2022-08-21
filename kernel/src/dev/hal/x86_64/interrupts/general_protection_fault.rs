use crate::*;
use panic;
use x86_64::structures::idt;
use core::arch::asm;
use super::*;

pub extern "x86-interrupt" fn general_protection_fault_handler(stack_frame: idt::InterruptStackFrame, error_code: u64) {
    unsafe {
        asm!("add rsp, 24
        pop rsp");
    }
    panic!("\n{:?}\nError code: {:#010x}\nStack frame: {:#?})", Exception::EXCEPTION_GENERAL_PROTECTION_FAULT, error_code, stack_frame);
}
