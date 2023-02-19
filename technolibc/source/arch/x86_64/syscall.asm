.section .text
.intel_syntax noprefix
.global _syscall

_syscall:
    mov rax, rdi  # syscall
    mov rdi, rsi  # arg0
    mov rsi, rdx  # arg1
    mov rdx, rcx  # arg2
    # mov r8, r8  # arg3
    syscall
    # return value in RAX
    ret