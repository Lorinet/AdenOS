use crate::{*, dev::hal::mem};
use alloc::{vec::Vec, boxed::Box};
use dev::hal::{task};
use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};

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
            self.processes[self.current_task].state = ctx;
            self.processes[self.current_task].save_state();
            self._next();
        }
        self.processes[self.current_task].restore_state();
        unsafe { task::restore_registers(&self.processes[self.current_task].state); }
    }

    fn _add_process(&mut self, process: task::Task) {
        self.processes.push(process);
    }

    fn _exec(&self, application: unsafe extern "C" fn()) {
        unsafe { task::Task::exec(application); }
    }

    fn _kexec(&self, application: unsafe fn()) {
        task::Task::kexec(application);
    }
}