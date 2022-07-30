pub mod breakpoint;
pub mod double_fault;
pub mod timer;

use super::pic;

#[derive(Debug)]
pub enum Exception {
    EXCEPTION_BREAKPOINT,
    EXCEPTION_DOUBLE_FAULT,
}