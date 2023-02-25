use crate::*;
use exec::*;
use collections::flat_map::*;
use alloc::{vec, vec::Vec, collections::vec_deque::VecDeque, collections::BTreeMap};
use dev::hal::{task, cpu};

static TICKS_PER_MILLISECOND: u32 = 1;

static mut SCHEDULER: Option<Scheduler> = None;
pub static DUMMY: &str = "hello";

pub struct Scheduler {
    processes: BTreeMap<u32, Vec<u32>>,
    threads: FlatMap<task::Task>,
    running_queue: Vec<u32>,
    suspended_queue: Vec<u32>,
    delta_queue: VecDeque<(u32, u32)>,
    current_thread_queue_index: u32,
    next_process_id: u32,
}

impl Scheduler {
    fn new() -> Scheduler {
        Scheduler {
            processes: BTreeMap::new(),
            threads: FlatMap::new(),
            running_queue: Vec::new(),
            suspended_queue: Vec::new(),
            delta_queue: VecDeque::new(),
            current_thread_queue_index: 0,
            next_process_id: 0,
        }
    }

    fn current_process(&self) -> u32 {
        self.threads[self.running_queue[self.current_thread_queue_index as usize]].process_id
    }

    fn current_thread(&self) -> u32 {
        self.running_queue[self.current_thread_queue_index as usize]
    }

    #[inline(always)]
    fn next(&mut self) {
        self.current_thread_queue_index += 1;
        if self.running_queue.len() <= self.current_thread_queue_index as usize {
            self.current_thread_queue_index = 0;
        }
    }

    #[inline(always)]
    fn context_switch(&mut self, current_context: Option<task::TaskContext>, on_timer_interrupt: bool) {
        if on_timer_interrupt {
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
        }
        if let Some(ctx) = current_context {
            let tid = self.running_queue[self.current_thread_queue_index as usize];
            let current_thread = &mut self.threads[tid];
            if current_thread.zombie {
                self.threads[tid].die();
                self.threads.remove(tid);
                self.running_queue.remove(self.current_thread_queue_index as usize);
                if self.current_thread_queue_index as usize >= self.running_queue.len() {
                    self.current_thread_queue_index = 0;
                }
            } else if current_thread.suspended {
                self.threads[tid].state = ctx;
                self.suspended_queue.push(tid);
                self.running_queue.remove(self.current_thread_queue_index as usize);
                if self.current_thread_queue_index as usize >= self.running_queue.len() {
                    self.current_thread_queue_index = 0;
                }
            } else {
                self.threads[tid].state = ctx;
                self.next();
            }
        }
        let tid = self.running_queue[self.current_thread_queue_index as usize];
        self.threads[tid].restore_state();
        unsafe { task::restore_registers(&self.threads[tid].state); }
    }

    fn add_thread(&mut self, process_id: u32, task: task::Task) -> Result<u32, Error> {
        if let None = self.processes.get(&process_id) {
            return Err(Error::EntryNotFound)
        }
        let tid = self.threads.insert_where_you_can(task.clone());
        self.processes.get_mut(&process_id).unwrap().push(tid);
        self.running_queue.push(tid);
        Ok(tid)
    }

    fn add_process(&mut self, task: task::Task) {
        let pid = self.get_new_process_id();
        self.processes.insert(pid, Vec::new());
        self.add_thread(pid, task).expect("Cannot find process just created.");
    }

    fn exec(&mut self, application: ExecutableInfo) -> Result<(), Error> {
        let pid = self.get_new_process_id();
        unsafe {
            self.add_process(task::Task::exec_new(application, pid)?);
            Ok(())
        }
    }

    fn kexec(&mut self, application: unsafe fn()) {
        let pid = self.get_new_process_id();
        unsafe {
            self.add_process(task::Task::kexec(application, pid));
        }
    }

    fn terminate_thread(&mut self, thread_id: u32) -> Result<(), Error> {
        if let Some(_) = self.threads.get(thread_id) {
            self.threads[thread_id].zombie = true;
            if let Some(joiner) = self.threads[thread_id].joiner {
                self.resume_thread(joiner)?;
            }
            task::trigger_context_switch();
            Ok(())
        } else {
            Err(Error::EntryNotFound)
        }
    }

    fn suspend_thread(&mut self, thread_id: u32) -> Result<(), Error> {
        if let Some(_) = self.threads.get(thread_id) {
            self.threads[thread_id].suspended = true;
            task::trigger_context_switch();
            Ok(())
        } else {
            Err(Error::EntryNotFound)
        }
    }

    fn resume_thread(&mut self, thread_id: u32) -> Result<(), Error> {
        let quin = self.suspended_queue.iter().position(|x| *x == thread_id).unwrap();
        if let Some(tid) = self.suspended_queue.get(quin) {
            self.suspended_queue.remove(quin);
            self.running_queue.push(thread_id);
            self.threads[thread_id].suspended = false;
            Ok(())
        } else {
            Err(Error::EntryNotFound)
        }
    }

    fn delay_thread(&mut self, thread_id: u32, milliseconds: u32) -> Result<(), Error> {
        cpu::disable_interrupts();
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
        self.suspend_thread(thread_id)?;
        Ok(())
    }

    fn join_thread(&mut self, joiner: u32, joinee: u32) -> Result<(), Error> {
        if let None = self.threads.get(joiner) {
            return Err(Error::EntryNotFound);
        }
        if let None = self.threads.get(joinee) {
            return Err(Error::EntryNotFound);
        }
        let _ = self.threads[joinee].joiner.insert(joiner);
        self.suspend_thread(joiner)?;
        Ok(())
    }
    
    fn terminate_process(&mut self, process_id: u32) -> Result<(), Error> {
        if let None = self.processes.get(&process_id) {
            return Err(Error::EntryNotFound);
        }
        let thrdlist = self.processes.get(&process_id).unwrap().clone();
        for thrd in thrdlist {
            self.terminate_thread(thrd);
        }
        self.processes.remove(&process_id);
        Ok(())
    }

    fn get_new_process_id(&mut self) -> u32 {
        while let Some(_) = self.processes.get(&self.next_process_id) {
            self.next_process_id += 1;
        }
        self.next_process_id
    }
}

pub fn init() {
    unsafe {
        SCHEDULER.insert(Scheduler::new());
    }
}

pub fn kexec(application: unsafe fn()) {
    unsafe { SCHEDULER.as_mut().unwrap().kexec(application); }
}

pub fn exec(application: ExecutableInfo) -> Result<(), Error> {
    unsafe { SCHEDULER.as_mut().unwrap().exec(application) }
}

#[inline(always)]
pub fn context_switch(current_context: Option<task::TaskContext>, timer_interrupt: bool) {
    unsafe { SCHEDULER.as_mut().unwrap().context_switch(current_context, timer_interrupt); }
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

pub fn delay_thread(thread_id: u32, milliseconds: u32) {
    unsafe {
        SCHEDULER.as_mut().unwrap().delay_thread(thread_id, milliseconds);
    }
}

pub fn terminate_thread(process: u32) {
    unsafe { SCHEDULER.as_mut().unwrap().terminate_thread(process); }
}

pub fn add_process(process: task::Task) {
    unsafe { SCHEDULER.as_mut().unwrap().add_process(process); }
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