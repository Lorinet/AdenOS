use crate::*;
use super::*;
use spin::*;
use dev::hal::cpu;
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use alloc::{ sync::Arc, collections::BTreeMap, task::Wake };
use core::task::{Waker, Context, Poll};

pub static KERNEL_TASKS: OnceCell<Mutex<BTreeMap<TaskId, Task>>> = OnceCell::uninit();
pub static KERNEL_TASK_QUEUE: OnceCell<Arc<ArrayQueue<TaskId>>> = OnceCell::uninit();
pub static KERNEL_WAKER_CACHE: OnceCell<Mutex<BTreeMap<TaskId, Waker>>> = OnceCell::uninit();

pub fn init() {
    KERNEL_TASKS.init_once(|| Mutex::new(BTreeMap::new()));
    KERNEL_TASK_QUEUE.init_once(|| Arc::new(ArrayQueue::new(128)));
    KERNEL_WAKER_CACHE.init_once(|| Mutex::new(BTreeMap::new()));
}

pub fn spawn(task: Task) {
    let task_id = task.id;
    let mut tasks = KERNEL_TASKS.try_get().expect("KERNEL_EXECUTOR_NOT_INITIALIZED").lock();
    if tasks.insert(task_id, task).is_some() {
        panic!("KERNEL_TASK_DOUBLE_ID\nTask ID: {:?}", task_id);
    }
    KERNEL_TASK_QUEUE.try_get().expect("KERNEL_EXECUTOR_NOT_INITIALIZED").push(task_id).expect("KERNEL_TASK_QUEUE_FULL");
}

fn run_ready_tasks() {
    let task_queue = KERNEL_TASK_QUEUE.try_get().expect("KERNEL_EXECUTOR_NOT_INITIALIZED");
    let mut tasks = KERNEL_TASKS.try_get().expect("KERNEL_EXECUTOR_NOT_INITIALIZED").lock();
    let mut waker_cache = KERNEL_WAKER_CACHE.try_get().expect("KERNEL_EXECUTOR_NOT_INITIALIZED").lock();
    while let Ok(task_id) = task_queue.pop() {
        let task = match tasks.get_mut(&task_id) {
            Some(task) => task,
            None => continue, // task no longer exists
        };
        let waker = waker_cache
            .entry(task_id)
            .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
        let mut context = Context::from_waker(waker);
        match task.poll(&mut context) {
            Poll::Ready(()) => {
                // task done -> remove it and its cached waker
                tasks.remove(&task_id);
                waker_cache.remove(&task_id);
            }
            Poll::Pending => {}
        }
    }
}

pub fn run() -> ! {
    loop {
        run_ready_tasks();
        let task_queue = KERNEL_TASK_QUEUE.try_get().expect("\nKERNEL_EXECUTOR_NOT_INITIALIZED");
        cpu::disable_interrupts();
        if task_queue.is_empty() {
            cpu::enable_interrupts();
            cpu::halt();
        } else {
            cpu::enable_interrupts();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("KERNEL_TASK_QUEUE_FULL");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}