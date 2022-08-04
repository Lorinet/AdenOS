use super::*;
use x86_64;
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;
use x86_64::instructions::tlb;
use x86_64::registers::model_specific::EferFlags;
use x86_64::registers::model_specific::LStar;
use x86_64::registers::segmentation::DS;
use x86_64::structures::idt;
use x86_64::structures::tss;
use x86_64::structures::gdt;
use x86_64::instructions;
use x86_64::instructions::tables;
use x86_64::instructions::segmentation;
use x86_64::registers::{model_specific::{Efer, Star}};
use x86_64::instructions::segmentation::Segment;
use lazy_static::lazy_static;
use core::arch::asm;
use alloc::{self, alloc::Layout};

const DOUBLE_FAULT_IST_INDEX: u16 = 0;

#[derive(Copy, Clone)]
struct Selectors {
    kernel_code_selector: gdt::SegmentSelector,
    kernel_data_selector: gdt::SegmentSelector,
    tss_selector: gdt::SegmentSelector,
    user_code_selector: gdt::SegmentSelector,
    user_data_selector: gdt::SegmentSelector,
}

static mut IDT: idt::InterruptDescriptorTable = idt::InterruptDescriptorTable::new();

lazy_static! {
    static ref TSS: tss::TaskStateSegment = {
        let mut tss = tss::TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = x86_64::VirtAddr::from_ptr(unsafe { &STACK });
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };

    static ref GDT: (gdt::GlobalDescriptorTable, Selectors) = {
        let mut gdt = gdt::GlobalDescriptorTable::new();
        let kernel_code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        let kernel_data_selector = gdt.add_entry(gdt::Descriptor::kernel_data_segment());
        let tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&TSS));
        let user_data_selector = gdt.add_entry(gdt::Descriptor::user_data_segment());
        let user_code_selector = gdt.add_entry(gdt::Descriptor::user_code_segment());
        (gdt, Selectors { kernel_code_selector, kernel_data_selector, tss_selector, user_code_selector, user_data_selector })
    };
}

pub fn init() {
    println!("Initializing CPU...");
    println!("Loading GDT...");
    GDT.0.load();
    unsafe {
        segmentation::CS::set_reg(GDT.1.kernel_code_selector);
        tables::load_tss(GDT.1.tss_selector);
        let handler_addr = system_call_handler as *const () as u64;
        asm!("\
        xor rdx, rdx
        mov rax, 0x200
        wrmsr",
        in("rcx") 0xc0000084 as u32);
        LStar::write(VirtAddr::new(handler_addr));
        Star::write(GDT.1.user_code_selector, GDT.1.user_data_selector, GDT.1.kernel_code_selector, GDT.1.kernel_data_selector).expect("GDT_CONFIG_FAILURE");
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }
    println!("Loading IDT...");
    unsafe {
        IDT.breakpoint.set_handler_fn(interrupts::breakpoint::breakpoint_handler);
        IDT.double_fault.set_handler_fn(interrupts::double_fault::double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        IDT.page_fault.set_handler_fn(interrupts::page_fault::page_fault_handler);
        IDT.general_protection_fault.set_handler_fn(interrupts::general_protection_fault::general_protection_fault_handler);
        IDT.stack_segment_fault.set_handler_fn(interrupts::stack_segment_fault::stack_segment_fault_handler);
        IDT.segment_not_present.set_handler_fn(interrupts::segment_not_present::segment_not_present_handler);
        IDT.invalid_tss.set_handler_fn(interrupts::invalid_tss::invalid_tss_handler);
        IDT.debug.set_handler_fn(interrupts::debug::debug_handler);
        IDT[interrupts::HardwareInterrupt::Timer.as_usize()].set_handler_fn(interrupts::timer::timer_handler);
        IDT.load();
    }
}

pub unsafe fn enter_user_mode(code_addr: usize, stack_addr: usize) {
    serial_println!("\n\n\n\n\n\n\n\n\n\nUSER MODE ENTERING:\n\n\n\n");
    let (mut cs, mut ds) = (GDT.1.user_code_selector, GDT.1.user_data_selector);
    cs.0 |= PrivilegeLevel::Ring3 as u16;
    ds.0 |= PrivilegeLevel::Ring3 as u16;
    DS::set_reg(ds);
    let (cs_idx, ds_idx) = (cs.0, ds.0);
    tlb::flush_all();
    asm!("\
    push rax   // stack segment
    push rsi   // rsp
    push 0x200 // rflags (only interrupt bit set)
    push rdx   // code segment
    push rdi   // ret to virtual addr
    iretq",
    in("rdi") code_addr,
    in("rsi") stack_addr,
    in("rdx") cs_idx, 
    in("rax") ds_idx);
}

unsafe fn system_call_handler() {
    asm!("\
    push rcx // backup registers for sysretq
    push r11
    push rbp // save callee-saved registers
    push rbx
    push r12
    push r13
    push r14
    push r15
    mov rbp, rsp // save rsp
    sub rsp, 0x400 // make some room in the stack
    push rax // backup syscall params while we get some stack space
    push rdi
    push rsi
    push rdx
    push r10");
    let syscall_stack = alloc::alloc::alloc(Layout::from_size_align_unchecked(0x1000, 0x1000));
    asm!("\
    pop r10 // restore syscall params to their registers
    pop rdx
    pop rsi
    pop rdi
    pop rax
    mov rsp, r12 // move our stack to the newly allocated one
    sti // enable interrupts",
    in("r12") syscall_stack);
    let syscall: u64;
    let arg0: u64;
    let arg1: u64;
    let arg2: u64;
    let arg3: u64;
    asm!("nop", out("rax") syscall, out("rdi") arg0, out("rsi") arg1, out("rdx") arg2, out("r10") arg3);
    println!("syscall {:x} {} {} {} {}", syscall, arg0, arg1, arg2, arg3);
    let retval: i64 = 0; // placeholder for the syscall's return value which we need to save and then return in rax
    asm!("mov rbx, {x} // save return value into rbx so that it's maintained through free
          cli", x = in(reg) retval);
    drop(syscall_stack); // we can now drop the syscall temp stack
    asm!("\
        mov rax, rbx // restore syscall return value from rbx to rax
        mov rsp, rbp // restore rsp from rbp
        pop r15 // restore callee-saved registers
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp // restore stack and registers for sysretq
        pop r11
        pop rcx
        sysretq // back to userland");
}

pub fn trigger_breakpoint() {
    instructions::interrupts::int3();
}

pub fn atomic_no_interrupts<F>(f: F)
where F: FnOnce() {
    disable_interrupts();
    f();
    enable_interrupts();
}

pub fn halt() {
    instructions::hlt();
}

pub fn grinding_halt() -> ! {
    cpu::disable_interrupts();
    loop {
        instructions::hlt();
    }
}

pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

pub fn disable_interrupts() {
    instructions::interrupts::disable();
}

pub fn pic_end_of_interrupt(int: interrupts::HardwareInterrupt) {
    pic::end_of_interrupt(int);
}

pub fn register_interrupt_handler(int: interrupts::HardwareInterrupt, handler: extern "x86-interrupt" fn(idt::InterruptStackFrame)) {
    unsafe {
        IDT[int.as_usize()].set_handler_fn(handler);
    }
}