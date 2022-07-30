use crate::*;
use dev::input::keyboard;
use dev::StaticDevice;
use sysinfo;
use dev;
use dev::hal::cpu;

pub fn run_kernel() -> ! {
    
    kernel_console::set_color(console::Color::LightGray, console::Color::Black);
    kernel_console::clear_screen();
    init_system();
    loop {
        cpu::halt();
    }
}

fn init_system() {
    println!("Neutrino Core OS [Version {}]", sysinfo::NEUTRINO_VERSION);
    println!("Setting up stuff like interrupts and shit...");
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::hal::cpu::init();
    println!("System init done!");
}

fn test_input_keyboard(key: keyboard::Key) {
    if let keyboard::Key::Unicode(ch) = key {
        print!("{}", ch);
    }
}

