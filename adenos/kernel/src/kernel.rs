use crate::*;
use crate::dev::hal::mem::page_mapper;
use crate::exec::thread;
use dev::*;
use infinity::allocator;
use crate::dev::*;
use crate::ipc::*;
use file;
use namespace::{self, *};
use alloc::boxed::Box;
use dev::input::keyboard;
use sysinfo;
use dev;
use dev::hal::*;
use async_task::*;
use crate::exec::scheduler;
use alloc::string::{String, ToString};

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
    infinity::connect_system_call_handler(syscall::system_call);
    early_print!("Linfinity Technologies AdenOS [Version {}]\n", sysinfo::ADEN_VERSION);
    dev::hal::init();
    early_print!("[{} MB Memory Available]\n", unsafe { mem::FREE_MEMORY } / 1048576 + 1);
    println!("");
    scheduler::init();
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
    let file = file::File::open(String::from("/Files/adenfs/main.elf"))?;
    let exe_inf = exec::elf::ELFLoader::load_executable(file.id)?;
    unsafe {
        scheduler::exec(exe_inf.clone(), &[])?;
    }
    serial_println!("{:#x?}", exe_inf);

    kernel_console::set_color(ConsoleColor::BrightBlue,  ConsoleColor::BrightBlack);
    kernel_executor::init();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    scheduler::kexec(kernel_executor::run, &[]);
    //scheduler::kexec(test_kernel_thread_with_ipc_recv, &[]);
    //scheduler::kexec(test_kernel_thread_with_ipc_send, &[]);
    thread::spawn(|| test_kernel_thread_clean());
    cpu::enable_scheduler();
    //kernel_executor::run();
    loop {}
    Ok(())
}

fn test_input_keyboard(key: keyboard::Key) {
    if let keyboard::Key::Unicode(ch) = key {
        print!("{}", ch);
    }
}

extern "C" fn test_kernel_thread_with_ipc_recv(argc: usize, argv: *const u8) {
    let pid = syscall::_get_process_id() as u32;
    println!("PID: {}", pid);
    let mq = namespace::register_resource(MessageChannel::new("hello".to_string(), Box::new(MessageQueue::new(pid, ipc::Endpoint::Any, 128))));
    for (_, dev) in namespace::namespace().iter_mut_bf() {
        if let Some(dev) = dev {
            println!("{}", dev.resource_path_string());
        }
    }
    loop {
        while mq.available() == 0 {}
        let mesg = mq.receive().unwrap();
        print!("MESSAGE: ");
        for b in mesg.bytes {
            print!("{}", b as char);
        }
        println!();
    }
    syscall::_exit();
    loop {}
}

extern "C" fn test_kernel_thread_with_ipc_send() {
    let mq = namespace::get_resource::<MessageQueue>(String::from("Processes/1/MessageQueues/0")).unwrap();
    for i in 0..128000 {
        mq.send(Message::new((String::from("Hello world") + i.to_string().as_str()).as_bytes()));
    }
    syscall::_exit();
    loop {}
}

fn test_kernel_thread_clean() {
    serial_println!("aaa");
    loop {
        serial_println!("{} Spawning", scheduler::current_thread());
        let thrd = thread::spawn(|| test_kernel_thread_clean_task());
        serial_println!("{} Joining", scheduler::current_thread());
        thrd.unwrap().join().unwrap();
        serial_println!("{} Joined", scheduler::current_thread());
        thread::sleep(3).unwrap();
    }
}

fn test_kernel_thread_clean_task() {
    /*println!("Wating 0.2s");
    thread::sleep(20);
    println!("Finished");*/
    println!("Payload");
}