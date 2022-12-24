#![cfg(target_arch = "x86_64")]

use crate::{*, dev::Device};
use alloc::{string::String, vec};

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
        namespace::register_resource(kernel_console::EARLY_FRAMEBUFFER.take().unwrap());
        let fb = kernel_console::FRAMEBUFFER.insert(namespace::get_resource(String::from("/Devices/Framebuffer/VesaVbeFramebuffer")).unwrap());
        namespace::register_resource(kernel_console::EARLY_KERNEL_CONSOLE.take().unwrap());
        let _ = kernel_console::KERNEL_CONSOLE.insert(namespace::get_resource(String::from("Devices/Character/FramebufferConsole")).unwrap());
        kernel_console::KERNEL_CONSOLE.as_mut().unwrap().framebuffer = fb;
        //kernel_console::FRAMEBUFFER.as_mut().unwrap().init_device().unwrap();
    }
}