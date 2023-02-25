use crate::*;
use super::*;
use dev::hal::task;

pub struct Thread {
    thread_id: Option<u32>,
    function: fn(),
}

impl Thread {
    pub fn new(function: fn()) -> Thread {
        Thread {
            thread_id: None,
            function,
        }
    }

    pub fn run(&mut self) {
        unsafe {
            if let Ok(tid) = scheduler::add_thread(scheduler::current_process(), task::Task::kexec(self.function, scheduler::current_process())) {
                let _ = self.thread_id.insert(tid);
            }
        }
    }

    pub fn join(&self) -> Result<(), Error> {
        if let Some(tid) = self.thread_id {
            scheduler::join_thread(scheduler::current_thread(), tid)
        } else {
            Err(Error::EntryNotFound)
        }
    }
}

pub fn sleep(milliseconds: u32) {
    scheduler::delay_thread(scheduler::current_thread(), milliseconds);
}

pub fn exit() -> ! {
    scheduler::terminate_thread(scheduler::current_thread());
    loop {}
}