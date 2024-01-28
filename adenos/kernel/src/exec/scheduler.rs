use crate::*;
use exec::*;
use collections::flat_map::*;
use alloc::{vec, vec::Vec, collections::vec_deque::VecDeque, collections::BTreeMap};
use dev::hal::{task, cpu};

static TICKS_PER_MILLISECOND: u32 = 1;

static mut SCHEDULER: Option<Scheduler> = None;
pub static DUMMY: &str = "hello";

pub struct Scheduler {
    processes: BTreeMap<u32, task::Process>,
    threads: FlatMap<task::Task>,
    running_queue: VecDeque<u32>,
    suspended_queue: VecDeque<u32>,
    delta_queue: VecDeque<(u32, u32)>,
    current_thread_queue_index: u32,
    next_process_id: u32,
    in_context_switch_currently: bool,
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            processes: BTreeMap::new(),
            threads: FlatMap::new(),
            running_queue: VecDeque::new(),
            suspended_queue: VecDeque::new(),
            delta_queue: VecDeque::new(),
            current_thread_queue_index: 0,
            next_process_id: 0,
            in_context_switch_currently: false,
        }
    }

    fn lock(&self) {
        if !self.in_context_switch_currently {
            cpu::disable_interrupts()
        }
    }

    fn unlock(&self) {
        if !self.in_context_switch_currently {
            cpu::enable_interrupts()
        }
    }

    fn current_process(&self) -> u32 {
        self.lock();
        let pid = self.threads[self.running_queue[self.current_thread_queue_index as usize]].process_id;
        self.unlock();
        pid
    }

    fn current_thread(&self) -> u32 {
        self.lock();
        let tid = self.running_queue[self.current_thread_queue_index as usize];
        self.unlock();
        tid
    }

    #[inline(always)]
    fn next(&mut self) {
        self.current_thread_queue_index += 1;
        if self.running_queue.len() <= self.current_thread_queue_index as usize {
            self.current_thread_queue_index = 0;
        }
    }

    #[inline(always)]
    fn context_switch(&mut self, current_context: Option<task::TaskContext>) {
        self.in_context_switch_currently = true;
        // MUST happen on timer interrupt to keep delay queue accurate
        let mut dequeue_delta = false;
        let mut resume_thread = 0;
        if let Some((tid, tick)) = self.delta_queue.get_mut(0) {
            if *tick == 0 {
                resume_thread = *tid;
                dequeue_delta = true;
            } else {
                *tick -= 1;
            }
        }
        if dequeue_delta {
            self.delta_queue.pop_front();
            self.resume_thread(resume_thread);
        }
        if let Some(ctx) = current_context {
            let tid = self.running_queue[self.current_thread_queue_index as usize];
            let current_thread = &mut self.threads[tid];
            if current_thread.zombie {
                let pid = self.threads[tid].process_id;
                self.threads[tid].die();
                let index = self.processes[&pid].threads.iter().position(|&dt| dt == tid).unwrap();
                self.processes.get_mut(&pid).unwrap().threads.remove(index);
                self.threads.remove(tid);
                self.running_queue.remove(self.current_thread_queue_index as usize);
                if self.processes[&pid].threads.len() == 0 {
                    // remove process if no threads left in it
                    self.processes[&pid].die();
                    self.processes.remove(&pid);
                }
                if self.current_thread_queue_index as usize >= self.running_queue.len() {
                    self.current_thread_queue_index = 0;
                }
            } else if current_thread.suspended {
                self.threads[tid].state = ctx;
                self.suspended_queue.push_back(tid);
                self.running_queue.remove(self.current_thread_queue_index as usize);
                if self.current_thread_queue_index as usize >= self.running_queue.len() {
                    self.current_thread_queue_index = 0;
                }
            } else {
                self.threads[tid].state = ctx;
                self.next();
            }
        }
        if self.running_queue.len() > 0 {
            let tid = self.running_queue[self.current_thread_queue_index as usize];
            self.threads[tid].restore_state();
            self.in_context_switch_currently = false;
            unsafe { task::restore_registers(&self.threads[tid].state); }
        }
        self.in_context_switch_currently = false;
    }

    fn add_thread(&mut self, process_id: u32, task: task::Task) -> Result<u32, Error> {
        self.lock();
        if let None = self.processes.get(&process_id) {
            self.unlock();
            return Err(Error::EntryNotFound)
        }
        let tid = self.threads.insert_where_you_can(task.clone());
        self.threads[tid].suspended = true;
        self.processes.get_mut(&process_id).unwrap().threads.push(tid);
        self.suspended_queue.push_back(tid);
        self.unlock();
        Ok(tid)
    }

    fn add_process(&mut self, task: task::Process) -> Result<u32, Error> {
        self.lock();
        let pid = self.get_new_process_id();
        self.processes.insert(pid, task);
        self.unlock();
        Ok(pid)
    }

    fn exec(&mut self, application: ExecutableInfo, argv: &[*const u8]) -> Result<(), Error> {
        self.lock();
        let pid = self.get_new_process_id();
        self.unlock();
        unsafe {
            task::Task::exec(application, pid, argv.len() as u32, argv.as_ptr() as *const *const u8)?;
        }
        Ok(())
    }

    fn kexec(&mut self, application: unsafe extern "C" fn(u32, *const *const u8), argv: &[*const u8]) -> Result<(), Error> {
        self.lock();
        let pid = self.get_new_process_id();
        self.unlock();
        unsafe {
            task::Task::kexec(application, pid, argv.len() as u32, argv.as_ptr() as *const *const u8)?;
        }
        Ok(())
    }

    fn terminate_thread(&mut self, thread_id: u32) -> Result<(), Error> {
        self.lock();
        if let Some(_) = self.threads.get(thread_id) {
            self.threads[thread_id].zombie = true;
            if let Some(joiner) = self.threads[thread_id].joiner {
                self.resume_thread(joiner)?;
            }
            task::trigger_context_switch();
            Ok(())
        } else {
            self.unlock();
            Err(Error::EntryNotFound)
        }
    }

    fn suspend_thread(&mut self, thread_id: u32) -> Result<(), Error> {
        self.lock();
        if let Some(_) = self.threads.get(thread_id) {
            self.threads[thread_id].suspended = true;
            task::trigger_context_switch();
            Ok(())
        } else {
            self.unlock();
            Err(Error::EntryNotFound)
        }
    }

    fn resume_thread(&mut self, thread_id: u32) -> Result<(), Error> {
        self.lock();
        if let Some(quin) = self.suspended_queue.iter().position(|x| *x == thread_id) {
            if let Some(tid) = self.suspended_queue.get(quin) {
                self.suspended_queue.remove(quin);
                self.running_queue.push_back(thread_id);
                self.threads[thread_id].suspended = false;
                self.unlock();
                return Ok(())
            }
        }
        self.unlock();
        Err(Error::EntryNotFound)
    }

    fn delay_thread(&mut self, thread_id: u32, milliseconds: u32) -> Result<(), Error> {
        self.lock();
        let mut delta = milliseconds * TICKS_PER_MILLISECOND;
        let mut i = 0;
        while let Some((_, prev)) = self.delta_queue.get_mut(i) {
            if delta > *prev {
                delta -= *prev;
                i += 1;
            } else {
                *prev -= delta;
                break;
            }
        }
        self.delta_queue.insert(i, (thread_id, delta));
        self.unlock();
        self.suspend_thread(thread_id)?;
        Ok(())
    }

    fn join_thread(&mut self, joiner: u32, joinee: u32) -> Result<(), Error> {
        self.lock();
        if let None = self.threads.get(joiner) {
            self.unlock();
            return Err(Error::EntryNotFound);
        }
        if let None = self.threads.get(joinee) {
            self.unlock();
            return Err(Error::EntryNotFound);
        }
        let _ = self.threads[joinee].joiner.insert(joiner);
        self.unlock();
        self.suspend_thread(joiner)?;
        Ok(())
    }
    
    fn terminate_process(&mut self, process_id: u32) -> Result<(), Error> {
        self.lock();
        if let None = self.processes.get(&process_id) {
            self.unlock();
            return Err(Error::EntryNotFound);
        }
        let thrdlist = self.processes.get(&process_id).unwrap().threads.clone();
        for thrd in thrdlist {
            self.threads[thrd].zombie = true;
        }
        task::trigger_context_switch();
        Ok(())
    }

    fn get_new_process_id(&mut self) -> u32 {
        self.lock();
        while let Some(_) = self.processes.get(&self.next_process_id) {
            self.next_process_id += 1;
        }
        self.unlock();
        self.next_process_id
    }
}

pub fn init() {
    unsafe {
        SCHEDULER.insert(Scheduler::new());
    }
}

pub fn kexec(application: unsafe extern "C" fn(u32, *const *const u8), argv: &[*const u8]) {
    unsafe { SCHEDULER.as_mut().unwrap().kexec(application, argv); }
}

pub fn exec(application: ExecutableInfo, argv: &[*const u8]) -> Result<(), Error> {
    unsafe { SCHEDULER.as_mut().unwrap().exec(application, argv) }
}

#[inline(always)]
pub fn context_switch(current_context: Option<task::TaskContext>) {
    unsafe { SCHEDULER.as_mut().unwrap().context_switch(current_context); }
}

#[inline(always)]
pub fn next() {
    unsafe { SCHEDULER.as_mut().unwrap().next(); }
}

#[inline(always)]
pub fn current_thread() -> u32 {
    unsafe {
        SCHEDULER.as_mut().unwrap().current_thread() as u32
    }
}

#[inline(always)]
pub fn current_process() -> u32 {
    unsafe {
        SCHEDULER.as_mut().unwrap().current_process()
    }
}

pub fn delay_thread(thread_id: u32, milliseconds: u32) -> Result<(), Error> {
    unsafe {
        SCHEDULER.as_mut().unwrap().delay_thread(thread_id, milliseconds)
    }
}

pub fn resume_thread(thread_id: u32) -> Result<(), Error> {
    unsafe {
        SCHEDULER.as_mut().unwrap().resume_thread(thread_id)
    }
}

pub fn terminate_thread(thread: u32) {
    unsafe { SCHEDULER.as_mut().unwrap().terminate_thread(thread); }
}

pub fn terminate_process(process: u32) {
    unsafe { SCHEDULER.as_mut().unwrap().terminate_process(process); }
}

pub fn add_process(process: task::Process) -> Result<u32, Error> {
    unsafe { SCHEDULER.as_mut().unwrap().add_process(process) }
}

pub fn add_thread(process_id: u32, task: task::Task) -> Result<u32, Error> {
    unsafe {
        SCHEDULER.as_mut().unwrap().add_thread(process_id, task)
    }
}

pub fn join_thread(joiner: u32, joinee: u32) -> Result<(), Error> {
    unsafe {
        SCHEDULER.as_mut().unwrap().join_thread(joiner, joinee)
    }
}

pub fn get_process(process_id: u32) -> Result<&'static task::Process, Error> {
    unsafe {
        if let Some(proc) = SCHEDULER.as_mut().unwrap().processes.get(&process_id) {
            return Ok(proc);
        }
        Err(Error::EntryNotFound)
    }
}