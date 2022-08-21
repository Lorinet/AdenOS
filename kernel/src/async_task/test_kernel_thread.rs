use crate::*;

pub fn test_kernel_thread() {
    loop {
        no_preempt_println!("This thread is running in kernel mode!");
    }
}