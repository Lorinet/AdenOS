# NeutrinoOS

## Portable, cross-platform OS for smart devices written in Rust

Build your app once, run it anywhere! <s>NeutrinoOS uses a JIT compiler to compile LLVM bitcode to the platform it's running on</s>

The operating system is currently at the early stages of development.

## Platform support

- x86_64
- <s>AARCH64</s>
- <s>ARMv7</s>
- <s>ARMv6-M</s>

## Build

NeutrinoOS uses a custom build tool called `n` for building, running, testing and debugging the OS. For the build to work, you need nightly Rust enabled.

### Build n

```
cd n
cargo install --path .
```

### Build NeutrinoOS

```
cd kernel
n
```

### Run NeutrinoOS

```
cd kernel
```

This requires `qemu-system-x86_64` to be installed on your system.

`n` can run NeutrinoOS using KVM or without using KVM. For KVM, use `n kvm`, and for non-KVM, use `n run`.

### Debug

```
n debug
```

This runs QEMU with `-s -S`, and you can attach GDB to `localhost:1234` to debug it.

## Run on real hardware

```
cd kernel
```

Plug in a USB stick, and find its device path (e.g. `/dev/sdc`). Afterwards, run the following command:

```
n flash <device>
```

To boot it, enable CSM and Legacy Boot in the UEFI settings, and boot directly off the USB. NeutrinoOS currently supports BIOS only, but the `bootloader` crate has UEFI build too, but it is currently untested with NeutrinoOS.

## Features

- Paging and virtual memory
- Bitmap frame allocator
- Heap allocation with SLAB allocator
- Userspace and kernel threads (with preemptive multitasking)
- VESA VBE framebuffer (currently text-only, no GUI)
- Reading ACPI tables
- PCI enumeration

## Credits

https://github.com/phil-opp/blog_os

https://github.com/nikofil/rust-os
