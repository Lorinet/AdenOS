use crate::{*, task::scheduler};
use infinity::os::*;

#[no_mangle]
#[inline(always)]
#[allow(unused_variables)]
pub extern "C" fn system_call(syscall: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    //serial_println!("syscall {:x} {:x} {:x} {:x} {:x}", syscall, arg0, arg1, arg2, arg3);
    match syscall {
        SYSTEM_CALL_READ => 1,
        SYSTEM_CALL_WRITE => _write(arg0, arg1 as *const u8, arg2),
        SYSTEM_CALL_SEEK => 1,
        SYSTEM_CALL_GET_IO_HANDLE => 1,
        SYSTEM_CALL_EXIT => _exit(),
        _ => 1,
    }
}

fn _write(_handle: usize, _buffer: *const u8, _count: usize) -> usize {
    println!("Hello");
    //unsafe { kernel_console::KERNEL_CONSOLE.as_mut().unwrap().write(slice::from_raw_parts(buffer, count)); }
    0
}

fn _exit() -> usize {
    scheduler::terminate(scheduler::current_process());
    0
}