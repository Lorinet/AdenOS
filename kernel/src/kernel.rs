use crate::*;
use dev::input::keyboard;
use dev::StaticDevice;
use sysinfo;
use dev;
use dev::hal::*;
use task::*;

pub fn run_kernel() -> ! {
    
    kernel_console::set_color(console::Color::LightGray, console::Color::Black);
    kernel_console::clear_screen();
    init_system();
    loop {
        cpu::halt();
    }
}

fn init_system() {
    println!("Linfinity Technologies Neutrino Core OS [Version {}]", sysinfo::NEUTRINO_VERSION);
    dev::hal::init();
    kernel_executor::init();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    kernel_executor::spawn(Task::new(example_task()));
    kernel_executor::run();
    println!("System init done!");
}

fn test_input_keyboard(key: keyboard::Key) {
    if let keyboard::Key::Unicode(ch) = key {
        print!("{}", ch);
    }
}

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
}