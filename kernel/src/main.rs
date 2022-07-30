#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(neutrino_os::test::test_runner)]
#![reexport_test_harness_main = "test_main"]

use neutrino_os::*;
use core::panic::PanicInfo;

#[no_mangle]
extern "C" fn _start() -> ! {
    #[cfg(test)]
    test_main();
    kernel::run_kernel()
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    neutrino_os::panic::panic(info)
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    neutrino_os::panic::test_panic(info)
}