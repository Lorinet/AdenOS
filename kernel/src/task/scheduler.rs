use crate::*;
use alloc::vec::Vec;
use dev::hal::{task, cpu};
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SCHEDULER: Scheduler = Scheduler::new();
}

pub struct Scheduler {
    processes: Mutex<Vec<task::Task>>,
    current_task: Mutex<Option<usize>>,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            processes: Mutex::new(Vec::new()),
            current_task: Mutex::new(None),
        }
    }

    #[inline(always)]
    pub unsafe fn context_switch(&self, current_context: Option<*const task::TaskContext>) {
        let mut processes = self.processes.lock();
        let mut current_task = self.current_task.lock();
        let next_task = current_task.map_or_else(|| { 0 }, |pid| {
            processes[pid].save_state(current_context.unwrap());
            let mut next = pid + 1;
            if processes.len() >= next {
                next = 0;
            }
            next
        });
        current_task.replace(next_task);
        let proc_ref = &processes[current_task.unwrap()] as *const task::Task;
        drop(processes);
        drop(current_task);
        (*proc_ref).restore_state();
    }

    pub fn add_process(&self, process: task::Task) {
        println!("Adding process");
        self.processes.lock().push(process);
        println!("Added process");
    }

    pub fn exec(&self, application: unsafe fn()) {
        unsafe { task::Task::exec(application); }
    }
}