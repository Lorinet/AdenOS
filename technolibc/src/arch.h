#pragma once

#define SYSCALL_WRITE 0
#define SYSCALL_READ 1
#define SYSCALL_SEEK 2
#define SYSCALL_GET_IO_HANDLE 3
#define SYSCALL_EXIT 4

extern unsigned long _syscall(unsigned long syscall, unsigned long arg0, unsigned long arg1, unsigned long arg2, unsigned long arg3);