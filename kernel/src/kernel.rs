use crate::*;
use crate::dev::partition::PartitionTable;
use crate::dev::storage::AHCIDrive;
use crate::dev::{RandomRead, filesystem};
use crate::namespace::ResourceType;
use alloc::vec;
use console::ConsoleColor;
use userspace::*;
use namespace;
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
    early_print!("Linfinity Technologies AdenOS [Version {}]\n", sysinfo::ADEN_VERSION);
    dev::hal::init();
    early_print!("[{} MB Memory Available]\n", unsafe { mem::FREE_MEMORY } / 1048576 + 1);
    println!("");
    let devices = namespace::namespace().get_subtree(String::from("Devices")).unwrap();
    for (_, dev) in devices.iter_mut_bf() {
        if let Some(dev) = dev {
            if let ResourceType::Device(dev) = dev.unwrap() {
                match dev.init_device() {
                    Ok(()) => println!("{} initialized successfully", dev.resource_path_string()),
                    Err(err) => println!("{} initialization failed: {:?}", dev.resource_path_string(), err),
                };
            }
        }
    }
    for (_, dev) in namespace::namespace().get_subtree(String::from("Devices")).unwrap().iter_mut_bf() {
        if let Some(dev) = dev {
            println!("{}", dev.resource_path_string());
        }
    }
    let drive = namespace::subtree(String::from("/Devices/Storage/AHCI/Drive0"));
    for (name, part) in drive.unwrap().iter_mut_bf() {
        //println!("{}", name);
        println!("{:#x?}", namespace::cast_resource::<dev::partition::Partition>(part.unwrap()));
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