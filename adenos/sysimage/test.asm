global _start

section .text
_start:
    mov rax, 7
    mov rdi, _device
    syscall
    mov rbx, rax
    mov rcx, 10
.loop:
    mov rdi, rbx ; handle
    mov rax, 1
    mov rsi, _message
    mov rdx, 6
    mov r14, rcx
    syscall
    mov rcx, r14
    loop .loop

    mov rax, 4
    syscall
    ret

_device: db "/Devices/Character/KernelLogger", 0
_message: db "Hello", 0
