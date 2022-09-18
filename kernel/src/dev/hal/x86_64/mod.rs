#![cfg(target_arch = "x86_64")]

use crate::{*, dev::Device};

pub mod port;
pub mod interrupts;
pub mod task;
pub mod acpi;
pub mod pic;
pub mod apic;
pub mod cpu;
pub mod mem;
pub mod pci;

pub fn init() {
    early_print!("x86_64 ");
    cpu::init();
    pic::init();
    mem::init();
    acpi::init();
    unsafe {
        kernel_console::FRAMEBUFFER.as_mut().unwrap().init_device().unwrap();
    }
}