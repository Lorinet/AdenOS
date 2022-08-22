use crate::*;
use console::ConsoleColor;
use userspace::*;
use dev::input::keyboard;
use dev::StaticDevice;
use sysinfo;
use dev;
use dev::hal::*;
use async_task::*;
use crate::task::scheduler;

pub fn run_kernel() -> ! {
    
    kernel_console::set_color(ConsoleColor::White,  ConsoleColor::BrightBlack);
    kernel_console::clear_screen();
    init_system();
    loop {
        cpu::halt();
    }
}

fn init_system() {
    early_print!("Linfinity Technologies Neutrino Core OS [Version {}]\n", sysinfo::NEUTRINO_VERSION);
    dev::hal::init();
    early_print!("[{} MB Memory Available]\n", unsafe { mem::FREE_MEMORY } / 1048576 + 1);
    kernel_console::set_color(ConsoleColor::BrightBlue,  ConsoleColor::BrightBlack);
    kernel_executor::init();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    scheduler::exec(userspace_app_1);
    scheduler::kexec(kernel_executor::run);
    cpu::enable_scheduler();
    kernel_executor::run();
}

fn test_input_keyboard(key: keyboard::Key) {
    if let keyboard::Key::Unicode(ch) = key {
        print!("{}", ch);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
        scheduler::exec(userspace_app_1);
    }
}