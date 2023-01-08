use crate::*;
use crate::dev::hal::mem::page_mapper;
use crate::dev::partition::PartitionTable;
use crate::dev::storage::AHCIDrive;
use crate::dev::*;
use crate::namespace::ResourceType;
use alloc::vec;
use console::ConsoleColor;
use namespace;
use dev::input::keyboard;
use dev::StaticDevice;
use sysinfo;
use dev;
use dev::hal::*;
use async_task::*;
use crate::exec::scheduler;
use alloc::string::String;

pub fn run_kernel() -> ! {
    
    kernel_console::set_color(ConsoleColor::White,  ConsoleColor::BrightBlack);
    kernel_console::clear_screen();
    if let Err(err) = init_system() {
        panic!("{:?}", err);
    }
    loop {
        cpu::halt();
    }
}

fn init_system() -> Result<(), Error> {
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
    for (_, dev) in namespace::namespace().iter_mut_bf() {
        if let Some(dev) = dev {
            println!("{}", dev.resource_path_string());
        }
    }

    serial_println!("{:?}", page_mapper::translate_addr(0x4000000));
    let file = file::File::open(String::from("/Files/adenfs/main.elf"))?;
    let exe_inf = exec::elf::ELFLoader::load_executable(file.id)?;
    unsafe {
        task::Task::exec_new(exe_inf.clone())?;
    }
    serial_println!("{:#x?}", exe_inf);

    kernel_console::set_color(ConsoleColor::BrightBlue,  ConsoleColor::BrightBlack);
    kernel_executor::init();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    scheduler::kexec(kernel_executor::run);
    cpu::enable_scheduler();
    kernel_executor::run();
    Ok(())
}

fn test_input_keyboard(key: keyboard::Key) {
    if let keyboard::Key::Unicode(ch) = key {
        print!("{}", ch);
    }
}