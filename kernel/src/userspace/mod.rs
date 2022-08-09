use core::arch::asm;
#[allow(named_asm_labels)]
pub fn userspace_app_1() {
    unsafe {
        asm!("\
            mov rdi, 0
            prog1start:
            push 22
            mov rax, 515ca11ah
            mov rsi, 3
            mov rdx, 4
            mov r10, 5
            syscall
            inc rdi
            jmp prog1start
        ");
    }
}