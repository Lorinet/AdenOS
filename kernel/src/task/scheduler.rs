use crate::*;
use alloc::{vec::Vec};
use dev::hal::{task};

static mut SCHEDULER: Scheduler = Scheduler::new();
pub static DUMMY: &str = "hello";
pub struct Scheduler {
    processes: Vec<task::Task>,
    current_task: usize,
}

impl Scheduler {
    pub fn kexec(application: unsafe fn()) {
        unsafe { SCHEDULER._kexec(application); }
    }

    pub fn exec(application: unsafe extern "C" fn()) {
        unsafe { SCHEDULER._exec(application); }
    }

    #[inline(always)]
    pub fn context_switch(current_context: Option<task::TaskContext>) {
        unsafe { SCHEDULER._context_switch(current_context); }
    }

    #[inline(always)]
    pub fn next() {
        unsafe { SCHEDULER._next(); }
    }

    #[inline(always)]
    pub fn current_process() -> usize {
        unsafe {
            /*if SCHEDULER.current_task == 0 {
                SCHEDULER.processes.len() - 1
            } else {
                SCHEDULER.current_task - 1
            }*/
            SCHEDULER.current_task
        }
    }

    pub fn terminate(process: usize) {
        unsafe { SCHEDULER._terminate(process); }
    }

    pub fn add_process(process: task::Task) {
        unsafe { SCHEDULER._add_process(process); }
    }

    const fn new() -> Scheduler {
        Scheduler {
            processes: Vec::new(),
            current_task: 0,
        }
    }

    #[inline(always)]
    fn _next(&mut self) {
        self.current_task += 1;
        if self.processes.len() <= self.current_task {
            self.current_task = 0;
        }
    }

    #[inline(always)]
    fn _context_switch(&mut self, current_context: Option<task::TaskContext>) {
        if let Some(ctx) = current_context {
            let current_process = &mut self.processes[self.current_task];
            if current_process.zombie {
                self.processes[self.current_task].die();
                self.processes.remove(self.current_task);
                if self.current_task >= self.processes.len() {
                    self.current_task = 0;
                }
            } else {
                self.processes[self.current_task].state = ctx;
                self._next();
            }
        }
        self.processes[self.current_task].restore_state();
        unsafe { task::restore_registers(&self.processes[self.current_task].state); }
    }

    fn _add_process(&mut self, process: task::Task) {
        self.processes.push(process.clone());
    }

    fn _exec(&self, application: unsafe extern "C" fn()) {
        unsafe { task::Task::exec(application); }
    }

    fn _kexec(&self, application: unsafe fn()) {
        unsafe { task::Task::kexec(application) };
    }

    fn _terminate(&mut self, process: usize) {
        self.processes[process].zombie = true;
    }
}