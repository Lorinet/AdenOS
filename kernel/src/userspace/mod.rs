use core::arch::asm;
use crate::*;

#[no_mangle]
#[allow(named_asm_labels)]
pub unsafe fn userspace_app_1() {
    asm!("\
        push 0
        prog1start:
        mov rax, 515ca11ah
        pop rsi
        inc rsi
        push rsi
        mov rdi, 0
        mov rdx, 4
        mov r8, 5
        syscall
        jmp prog1start
    ");
}

#[no_mangle]
#[allow(named_asm_labels)]
pub unsafe fn userspace_app_2() {
    asm!("\
        push 0
        prog2start:
        mov rax, 515ca11bh
        pop rsi
        inc rsi
        push rsi
        mov rdi, 0
        mov rdx, 4
        mov r8, 5
        syscall
        jmp prog2start
    ");
}