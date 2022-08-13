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
use alloc::{alloc::{alloc, dealloc, Layout}};

const INTERRUPT_IST_INDEX: u16 = 0;
const SCHEDULER_INTERRUPT_IST_INDEX: u16 = 0;

#[derive(Copy, Clone)]
struct Selectors {
    kernel_code_selector: gdt::SegmentSelector,
    kernel_data_selector: gdt::SegmentSelector,
    tss_selector: gdt::SegmentSelector,
    user_code_selector: gdt::SegmentSelector,
    user_data_selector: gdt::SegmentSelector,
}

static mut IDT: idt::InterruptDescriptorTable = idt::InterruptDescriptorTable::new();
static mut SYSCALL_STACK: [u8; 0x4000] = [0; 0x4000];
static mut INTERRUPT_STACK: [u8; 0x4000] = [0; 0x4000];
static mut SCHEDULER_INTERRUPT_STACK: [u8; 0x4000] = [0; 0x4000];

lazy_static! {
    #[derive(Debug)]
    pub static ref TSS: tss::TaskStateSegment = {
        let mut tss = tss::TaskStateSegment::new();
        tss.interrupt_stack_table[INTERRUPT_IST_INDEX as usize] = unsafe { 
            VirtAddr::new(&INTERRUPT_STACK as *const u8 as u64 + 0x4000)
        };
        tss.interrupt_stack_table[SCHEDULER_INTERRUPT_IST_INDEX as usize] = unsafe { 
            VirtAddr::new(&SCHEDULER_INTERRUPT_STACK as *const u8 as u64 + 0x4000)
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
        IDT.breakpoint.set_handler_fn(interrupts::breakpoint::breakpoint_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.double_fault.set_handler_fn(interrupts::double_fault::double_fault_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.page_fault.set_handler_fn(interrupts::page_fault::page_fault_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.general_protection_fault.set_handler_fn(interrupts::general_protection_fault::general_protection_fault_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.stack_segment_fault.set_handler_fn(interrupts::stack_segment_fault::stack_segment_fault_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.segment_not_present.set_handler_fn(interrupts::segment_not_present::segment_not_present_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.debug.set_handler_fn(interrupts::debug::debug_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT[interrupts::HardwareInterrupt::Timer.as_usize()].set_handler_fn(interrupts::timer::timer_handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.load();
    }
    println!("IDT address: {:#018x}", unsafe { &IDT as *const _ as u64 });
    println!("TSS address: {:#018x}", unsafe { &TSS as *const _ as u64 });
}

pub unsafe fn enter_user_mode(code_addr: usize, stack_addr: usize) {
    serial_println!("\n\n\n\n\n\n\n\n\n\nUSER MODE ENTERING:\n\n\n\n");
    let (mut cs, mut ds) = (GDT.1.user_code_selector, GDT.1.user_data_selector);
    cs.0 |= PrivilegeLevel::Ring3 as u16;
    ds.0 |= PrivilegeLevel::Ring3 as u16;
    DS::set_reg(ds);
    let (cs_idx, ds_idx) = (cs.0, ds.0);
    tlb::flush_all();
    asm!("cli");
    IDT[interrupts::HardwareInterrupt::Timer.as_usize()].set_handler_addr(VirtAddr::new(task::timer_handler_save_context as u64)).set_stack_index(SCHEDULER_INTERRUPT_IST_INDEX);
    asm!("\
    push rax   // stack segment
    push rsi   // rsp
    push 0x200 // rflags (only interrupt bit set)
    push rdx   // code segment
    push rdi   // ret to virtual addr
    sti
    iretq",
    in("rdi") code_addr,
    in("rsi") stack_addr,
    in("rdx") cs_idx, 
    in("rax") ds_idx);
}

#[no_mangle]
unsafe extern "C" fn system_call_handler_hl() {
    // very sketchy code
    asm!("
        cli
        mov r9, rsp  // save user stack
        mov rsp, r12 // set syscall stack
        sti", in("r12") (&SYSCALL_STACK as *const u8 as u64));
    let syscall: u64;
    let param_0: u64;
    let param_1: u64;
    let param_2: u64;
    let param_3: u64;
    let user_mode_stack: u64;
    asm!("nop // load syscall parameters",
        out("rax") syscall,
        out("rdi") param_0,
        out("rsi") param_1,
        out("rdx") param_2,
        out("r8")  param_3,
        out("r9")  user_mode_stack);
    println!("syscall {:x} {:x} {} {} {}", syscall, param_0, param_1, param_2, param_3);
    asm!("cli
    mov rsp, r12 // set user mode stack
    sti", in("r12") user_mode_stack);
}

#[naked]
unsafe fn system_call_handler() {
    asm!("
    push rcx // sysretq rip
    push r11 // sysretq rflags
    push r15 // callee-saved registers
    push r14
    push r13
    push r12
    push rbp
    push rbx
    call system_call_handler_hl
    pop rbx
    pop rbp
    pop r12
    pop r13
    pop r14
    pop r15
    pop r11
    pop rcx
    sysretq
    ", options(noreturn));
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
        IDT[int.as_usize()].set_handler_fn(handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.load();
    }
}
