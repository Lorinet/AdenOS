#![no_std]
#![feature(naked_functions)]

pub mod arch;

pub mod os;
pub mod dev;
pub mod ipc;
pub mod error;
pub mod handle;
pub mod allocator;

extern crate alloc;

pub use error::*;

#[cfg(feature = "kernel_mode")]
pub fn connect_system_call_handler(handler: extern "C" fn(usize, usize, usize, usize, usize) -> isize) {
    unsafe {
        arch::_system_call_handler = handler;
    }
}