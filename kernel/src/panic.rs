use crate::*;
use dev;
use dev::PowerControl;
use kernel_console;
use console;
use core::panic::PanicInfo;
use dev::hal::cpu;

pub fn panic(info: &PanicInfo) -> ! {
    cpu::grinding_halt();
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