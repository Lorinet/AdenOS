#![cfg(target_arch = "x86_64")]
use crate::*;
use bootloader::{BootInfo, entry_point};

pub mod port;
pub mod cpu;
pub mod mem;
pub mod interrupts;
mod pic;

pub fn init() {
    println!("Initializing x86_64 HAL...");
    cpu::init();
    pic::init();
    unsafe { mem::init(); }
    println!("Platform initialization done!");
}