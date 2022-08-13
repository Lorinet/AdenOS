#![no_std]
#![cfg_attr(test, no_main)]
#![feature(naked_functions)]
#![feature(abi_x86_interrupt)]
#![feature(panic_info_message)]
#![feature(alloc_error_handler)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

pub mod dev;
pub mod test;
pub mod task;
pub mod panic;
pub mod kernel;
pub mod console;
pub mod sysinfo;
pub mod syscalls;
pub mod allocator;
pub mod userspace;
pub mod async_task;
pub mod kernel_console;

extern crate alloc;

#[cfg(test)]
#[cfg(target_arch = "x86_64")]
use bootloader::{BootInfo, entry_point};
#[cfg(test)]
#[cfg(target_arch = "x86_64")]
entry_point!(test_kernel_main);
#[cfg(test)]
#[cfg(target_arch = "x86_64")]
fn test_kernel_main(boot_info: &'static BootInfo) -> ! {
    unsafe { 
        dev::hal::mem::PHYSICAL_MEMORY_OFFSET = boot_info.physical_memory_offset.try_into().unwrap();
        dev::hal::mem::BOOT_MEMORY_MAP = Some(&boot_info.memory_map);
    }
    dev::hal::init();
    #[cfg(test)]
    test_main();
    loop {
        dev::hal::cpu::halt();
    }
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    panic::test_panic(info)
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("\nMEMORY_ALLOCATION_ERROR\n{:#?}", layout)
}