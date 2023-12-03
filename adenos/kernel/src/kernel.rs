use crate::*;
use crate::dev::hal::mem::page_mapper;
use crate::exec::thread;
use dev::*;
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
    let file = file::File::open(String::from("/Files/AdenOS/test.elf"))?;
    let exe_inf = exec::elf::ELFLoader::load_executable(file.id)?;
    scheduler::exec(exe_inf.clone())?;
    serial_println!("{:#x?}", exe_inf);

    kernel_console::set_color(ConsoleColor::BrightBlue,  ConsoleColor::BrightBlack);
    kernel_executor::init();
    dev::input::PS2KeyboardPIC8259::set_input_handler(test_input_keyboard);
    dev::input::PS2KeyboardPIC8259::init_device().unwrap();
    scheduler::kexec(kernel_executor::run);
    //scheduler::kexec(test_kernel_thread_with_ipc_recv);
    //scheduler::kexec(test_kernel_thread_with_ipc_send);
    //scheduler::kexec(test_kernel_thread_joiner);
    //scheduler::kexec(test_kernel_thread_killer);
    //scheduler::kexec(test_kernel_thread_clean);
    cpu::enable_scheduler();
    Ok(())
}

fn test_input_keyboard(key: keyboard::Key) {
    if let keyboard::Key::Unicode(ch) = key {
        print!("{}", ch);
    }
}

fn test_kernel_thread_with_ipc_recv() {
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

fn test_kernel_thread_with_ipc_send() {
    let mq = namespace::get_resource::<MessageQueue>(String::from("Processes/1/MessageQueues/0")).unwrap();
    for i in 0..128000 {
        mq.send(Message::new((String::from("Hello world") + i.to_string().as_str()).as_bytes()));
    }
    syscall::_exit();
    loop {}
}

fn test_kernel_thread_killer() {
    println!("You live for 8 seconds, then you die.");
    thread::sleep(8000);
    scheduler::terminate_process(2);
    println!("Done with ya!");
    scheduler::terminate_process(scheduler::current_process());
}

fn test_kernel_thread_joiner() {
    loop {
        println!("Creating new threads PID {}", scheduler::current_process());
        let mut thrd = thread::Thread::new(test_kernel_thread_joinee);
        let mut thrd2 = thread::Thread::new(test_kernel_thread_joinee2);
        thrd.run();
        thrd2.run();
        println!("Joining new thread");
        thrd.join();
        thrd2.join();
        println!("Done!");
    }
}

fn test_kernel_thread_joinee() {
    println!("0 Sleeping for 0.5sec...");
    thread::sleep(500);
    println!("0 Creating new thread");
    let mut thrd = thread::Thread::new(test_kernel_thread_joinee3);
    thrd.run();
    println!("0 Joining new thread");
    thrd.join();
    println!("0 Done!");
    thread::exit();
}

fn test_kernel_thread_joinee2() {
    println!("1 Sleeping for 1sec...");
    thread::sleep(1000);
    println!("1 Done sleeping");
    thread::exit();
}

fn test_kernel_thread_joinee3() {
    println!("2 Sleeping for 2sec...");
    thread::sleep(2000);
    println!("2 Done sleeping");
    thread::exit();
}

fn test_kernel_thread_clean() {
    loop {
        let mut thrd = thread::Thread::new(test_kernel_thread_clean_task);
        thrd.run();
        thread::sleep(500);
    }
}

fn test_kernel_thread_clean_task() {
    println!("Wating 0.2s");
    thread::sleep(200);
    println!("Finished");
    thread::exit();
}