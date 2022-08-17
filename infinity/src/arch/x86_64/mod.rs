use crate::*;
use core::arch::asm;

#[no_mangle]
#[inline(always)]
pub extern "C" fn _system_call(syscall: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> usize {
    let return_value: usize;
    unsafe {
        asm!("
        push rdi // syscall
        push rsi // arg0
        push rdx // arg1
        push rcx // arg2
        // arg3 is in r8
        pop rdx  // arg2
        pop rsi  // arg1
        pop rdi  // arg0
        pop rax  // syscall
        syscall
        mov {0}, rax", out(reg) return_value);
    }
    return_value
}