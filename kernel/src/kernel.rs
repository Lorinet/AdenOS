use crate::*;
use crate::dev::filesystem::PartitionTable;
use crate::dev::{RandomRead, filesystem};
use alloc::vec;
use console::ConsoleColor;
use userspace::*;
use dev::input::keyboard;
use dev::StaticDevice;
use sysinfo;
use dev;
use dev::hal::*;
use async_task::*;
use crate::task::scheduler;
use alloc::string::String;

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
    println!("");
    for (name, dev) in devices::device_tree().iter_mut_bf() {
        if let Some(dev) = dev {
            match dev.init_device() {
                Ok(()) => println!("{} initialized successfully", name),
                Err(err) => println!("{} initialization failed: {:?}", name, err),
            };
        }
    }
    for (dev, _) in devices::device_tree().iter_mut_bf() {
        println!("{}", dev);
    }
    let mbr = filesystem::mbr::MBRPartitionTable::read_partitions(devices::get_device::<dev::storage::AHCIDrive>(vec![String::from("Storage"), String::from("AHCI"), String::from("Drive1")])).unwrap();
    for part in mbr {
        println!("{:#x?}", part);
    }
    kernel_console::set_color(ConsoleColor::BrightBlue,  ConsoleColor::BrightBlack);
    kernel_executor::init();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    scheduler::exec(userspace_app_1);
    scheduler::exec(userspace_app_1);
    scheduler::exec(userspace_app_1);
    scheduler::exec(userspace_app_1);
    scheduler::exec(userspace_app_1);
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