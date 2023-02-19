#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(adenos::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use adenos::*;
use dev::*;
use core::panic::PanicInfo;

#[cfg(target_arch = "x86_64")]
use {
    dev::hal::*,
    dev::framebuffer::*,
    dev::char::*,
    bootloader::{BootInfo, entry_point}
};
#[cfg(target_arch = "x86_64")]
entry_point!(kernel_main);
#[cfg(target_arch = "x86_64")]
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    //loop {}
    unsafe {
        mem::PHYSICAL_MEMORY_OFFSET = boot_info.physical_memory_offset.into_option().unwrap();
        mem::BOOT_MEMORY_MAP = Some(&boot_info.memory_regions);
        let free_mem = boot_info.memory_regions.iter().map(|reg| reg.end - reg.start);
        let free_mem: u64 = free_mem.sum();
        mem::FREE_MEMORY = free_mem as usize;
        let bifb = boot_info.framebuffer.as_mut().unwrap();
        let bifbi = bifb.info();
        kernel_console::EARLY_FRAMEBUFFER = Some(VesaVbeFramebuffer::new(bifb.buffer_mut(), bifbi.horizontal_resolution, bifbi.vertical_resolution,
    match bifbi.pixel_format {
            bootloader::boot_info::PixelFormat::RGB => framebuffer::PixelFormat::RGB,
            bootloader::boot_info::PixelFormat::BGR => framebuffer::PixelFormat::BGR,
            bootloader::boot_info::PixelFormat::U8 => framebuffer::PixelFormat::Monochrome,
            _ => framebuffer::PixelFormat::RGB,
        }, bifbi.bytes_per_pixel, bifbi.stride));
        kernel_console::FRAMEBUFFER = Some(kernel_console::EARLY_FRAMEBUFFER.as_mut().unwrap());
        kernel_console::EARLY_KERNEL_CONSOLE = Some(FramebufferConsole::new(*kernel_console::FRAMEBUFFER.as_mut().unwrap()));
        kernel_console::KERNEL_CONSOLE = Some(kernel_console::EARLY_KERNEL_CONSOLE.as_mut().unwrap());
        acpi::RSDP_ADDRESS = *boot_info.rsdp_addr.as_ref().unwrap();
    }
    #[cfg(test)]
    test_main();
    kernel::run_kernel()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    adenos::panic::panic(info)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    adenos::panic::test_panic(info)
}
