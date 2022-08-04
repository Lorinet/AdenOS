#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(neutrino_os::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use neutrino_os::*;
use core::panic::PanicInfo;

#[cfg(target_arch = "x86_64")]
use {
    dev::hal::*,
    bootloader::{BootInfo, entry_point}
};
#[cfg(target_arch = "x86_64")]
entry_point!(kernel_main);
#[cfg(target_arch = "x86_64")]
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    unsafe { 
        dev::hal::mem::PHYSICAL_MEMORY_OFFSET = boot_info.physical_memory_offset.try_into().unwrap();
        dev::hal::mem::BOOT_MEMORY_MAP = Some(&boot_info.memory_map);
    }
    #[cfg(test)]
    test_main();
    kernel::run_kernel()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    #[cfg(target_arch = "x86_64")]
    kernel_console::KERNEL_CONSOLE.lock().disable_cursor();
    neutrino_os::panic::panic(info)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    neutrino_os::panic::test_panic(info)
}