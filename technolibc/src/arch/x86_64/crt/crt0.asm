.section .text
.intel_syntax noprefix
.global _start
.globl exit
_start:
	# Set up end of the stack frame linked list.
	mov rbp, 0
	mov rdi, [rsp]
	


    mov rbp, 0
    push rbp # rip=0
    push rbp # rbp=0
    mov rbp, rsp

	# We need those in a moment when we call main.
    push rsi
    push rdi

	# Prepare signals, memory allocation, stdio and such.
	# call initialize_standard_library

	# Run the global constructors.
	# call _init

	# Restore argc and argv.
    pop rdi
    pop rsi

	# Run main
	call main

	# Terminate the process with the exit code.
    mov rdi, rax
	call exit

spinloop:
	jmp spinloop # wait for AdenOS to terminate process
.size _start, . - _start