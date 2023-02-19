.section .text
.intel_syntax noprefix

.global _start
.global _init
.global _fini
.global _syscall
.global main

_start:
	mov rbp, 0 # set up stack frame
	mov rdi, [rsp]      # argc
	lea rsi, [rsp + 8]  # argv
	lea rdx, [rsp + 16] # envp

	push rdx # save main arguments
	push rsi
	push rdi

	call _init # global constructors from crti.asm

	pop rdi # restore main arguments
	pop rsi
	pop rdx

	call main

	push rax # save exit code

	call _fini # global destructors from crti.asm

	pop rsi # restore exit code
	mov rdi, 4 # syscall code for Exit
	call _syscall

spinloop:
	jmp spinloop # wait for AdenOS to terminate process
.size _start, . - _start