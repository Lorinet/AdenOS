use crate::*;
use dev;
use dev::PowerControl;
use kernel_console;
use console;
use core::fmt::Debug;
use core::panic::PanicInfo;
use dev::hal::cpu;

pub fn trigger_panic_exception(exception: &dyn Debug, stack_frame: &dyn Debug, error_code: u64) -> ! {
    panic!("\n{:?}\nError code: {:#010x}\nStack frame: {:#?})", exception, error_code, stack_frame);
}

pub fn panic(info: &PanicInfo) -> ! {
    kernel_console::set_color(console::Color::LightGray, console::Color::Blue);
    kernel_console::clear_screen();
    print!("\n");
    println!(" \\|/ ____ \\|/");
    println!(" \"@'/ xx \\`@\"   Neutrino Core OS");
    println!(" /_| \\__/ |_\\   Kernel Panic");
    println!("    \\__U_/");
    print!("\n");
    println!("{}", info);
    cpu::grinding_halt()
}

pub fn test_panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]");
    serial_println!("Error: {}\n", info);
    let mut exit = dev::power::QemuExit::new();
    exit.reboot()
}

pub fn test_should_panic(_: &PanicInfo) -> ! {
    serial_println!("[ok]");
    let mut exit = dev::power::QemuExit::new();
    exit.shutdown()
}