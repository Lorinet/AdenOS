#!/bin/bash
nasm test.asm -o test.o -f elf64
ld test.o -o test.elf -T link.ld
