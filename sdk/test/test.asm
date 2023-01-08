.intel_syntax noprefix
.text
.global main
main:
    mov rax, 1
    mov rdi, 0x12345678
    syscall
    mov rax, 4
    syscall
lp:
    jmp lp
    ret
