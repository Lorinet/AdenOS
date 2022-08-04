#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![reexport_test_harness_main = "test_main"]
#![test_runner(neutrino_os::test::test_runner)]

use neutrino_os::*;
use core::panic::PanicInfo;
use alloc::vec::Vec;
use alloc::boxed::Box;

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

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    neutrino_os::panic::test_panic(info)
}

#[test_case]
fn simple_allocation() {
    let heap_value_1 = Box::new(41);
    let heap_value_2 = Box::new(13);
    assert_eq!(*heap_value_1, 41);
    assert_eq!(*heap_value_2, 13);
}

#[test_case]
fn large_vec() {
    let n = 1000;
    let mut vec = Vec::new();
    for i in 0..n {
        vec.push(i);
    }
    assert_eq!(vec.iter().sum::<u64>(), (n - 1) * n / 2);
}


#[test_case]
fn many_boxes() {
    for i in 0..dev::hal::mem::KERNEL_HEAP_SIZE {
        let x = Box::new(i);
        assert_eq!(*x, i);
    }
}
