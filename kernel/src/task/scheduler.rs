use crate::*;
use alloc::vec::Vec;
use dev::hal::task::*;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref SCHEDULER: Scheduler = Scheduler::new();
}

pub struct Scheduler {
    processes: Mutex<Vec<Task>>,
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
    pub unsafe fn context_switch(&self, current_context: *const TaskContext) {
        self.current_task.lock().map(|pid| {
            let mut processes = self.processes.lock();
            processes[pid].save_state((*current_context).clone());
            let mut next = pid + 1;
            if processes.len() >= next {
                next = 0;
            }
            Some(next)
        });
        self.processes.lock()[self.current_task.lock().unwrap()].restore_state();
    }

    pub fn add_process(&self, process: Task) {
        self.processes.lock().push(process);
    }

    pub fn exec(&self, application: unsafe fn()) {
        unsafe { Task::exec(application); }
    }
}