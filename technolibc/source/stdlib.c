#include <arch.h>
#include <stdlib.h>

void exit(int code) {
    _syscall(SYSCALL_EXIT, code, 0, 0, 0);
}