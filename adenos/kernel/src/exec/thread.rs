use crate::*;
use super::*;
use alloc::{vec, string::ToString, boxed::Box};
use dev::hal::task;
use namespace::*;

#[derive(Copy, Clone, Debug)]
pub struct ThreadData<F, T>
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    dummy: u64,
    closure: Option<F>,
    result: Option<T>,
}

impl<F, T> ThreadData<F, T>
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    pub fn new(closure: F) -> ThreadData<F, T> {
        ThreadData {
            dummy: 0x12345678ABCDEF0,
            closure: Some(closure),
            result: None,
        }
    }

    pub fn run(&mut self) {
        let _ = self.result.insert((self.closure.take().unwrap())());
    }
}

pub struct Thread<F, T>
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    process_id: u32,
    thread_id: Option<u32>,
    thread_data: Box<ThreadData<F, T>>,
    is_open: bool,
}

impl<F, T> Resource for Thread<F, T>
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    fn resource_path(&self) -> Vec<String> {
        vec![String::from("Processes"), self.process_id.to_string(), String::from("Threads"), self.thread_id.unwrap().to_string()]
    }

    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open_state(&mut self, open: bool) {
        self.is_open = open
    }

    fn unwrap(&mut self) -> ResourceType {
        ResourceType::Other
    }
}

impl<F, T> Thread<F, T>
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    pub fn new(function: F) -> Thread<F, T> {
        let process_id = scheduler::current_process();
        Thread {
            process_id,
            thread_id: None,
            thread_data: Box::new(ThreadData::new(function)),
            is_open: false,
        }
    }

    pub fn run(&mut self) -> Result<(), Error> {
        let pntr = self.thread_data.as_ref() as *const _ as *const *const u8;
        let tid = scheduler::add_thread(self.process_id, unsafe { task::Task::kexec_thread(_thread_entry::<F, T>, self.process_id, 1, pntr)? })?;
        self.thread_id.insert(tid);
        scheduler::resume_thread(tid)?;
        Ok(())
    }

    pub fn join(&self) -> Result<(), Error> {
        if let None = self.thread_id {
            return Err(Error::EntryNotFound);
        }
        scheduler::join_thread(scheduler::current_thread(), self.thread_id.unwrap())?;
        Ok(())
    }
}

extern "C" fn _thread_entry<F, T>(argc: u32, argv: *const *const u8)
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    unsafe {
        let thread_data = (argv as *const _ as *mut ThreadData<F, T>).as_mut().unwrap();
        thread_data.run();
    }
    exit();
}

pub fn spawn<F, T>(f: F) -> Result<&'static mut Thread<F, T>, Error>
where F: FnOnce() -> T, F: Send + 'static, T: Send + 'static {
    let mut thrd = Thread::new(f);
    thrd.run()?;
    let r = namespace::register_resource(thrd);
    serial_println!("{} Registered", scheduler::current_thread());
    Ok(r)
}

pub fn sleep(milliseconds: u32) -> Result<(), Error> {
    scheduler::delay_thread(scheduler::current_thread(), milliseconds)
}

pub fn exit() -> ! {
    let resp = vec![String::from("Processes"), scheduler::current_process().to_string(), String::from("Threads"), scheduler::current_thread().to_string()];
    if let Err(err) = namespace::drop_resource_parts(resp.clone()) {
        serial_println!("NOT FOUND: {:?}", resp);
    }
    drop(resp);
    scheduler::terminate_thread(scheduler::current_thread());
    loop {}
}