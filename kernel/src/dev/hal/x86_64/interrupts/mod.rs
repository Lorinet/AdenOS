pub mod debug;
pub mod timer;
pub mod breakpoint;
pub mod double_fault;
pub mod general_protection_fault;
pub mod stack_segment_fault;
pub mod segment_not_present;
pub mod page_fault;

use super::pic;

#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum Exception {
    EXCEPTION_DEBUG,
    EXCEPTION_BREAKPOINT,
    EXCEPTION_DOUBLE_FAULT,
    EXCEPTION_PAGE_FAULT,
    EXCEPTION_GENERAL_PROTECTION_FAULT,
    EXCEPTION_STACK_SEGMENT_FAULT,
    EXCEPTION_SEGMENT_NOT_PRESENT,
    EXCEPTION_INVALID_TSS,
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HardwareInterrupt {
    Timer = pic::PIC_MASTER_OFFSET,
    Keyboard,
}

impl HardwareInterrupt {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}