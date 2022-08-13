#![cfg(target_arch = "x86_64")]

use crate::*;
pub mod port;
pub mod cpu;
pub mod mem;
pub mod interrupts;
pub mod task;
pub mod pic;

pub fn init() {
    early_print!("Initializing x86_64 HAL...\n");
    cpu::init();
    pic::init();
    mem::init();
    println!("Platform initialization done!");
}