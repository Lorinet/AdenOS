use crate::*;
use dev::hal::{interrupts, pic, task};
use x86_64::registers;
use x86_64::structures::gdt::SegmentSelector;
use x86_64::structures::idt::InterruptStackFrame;
use crate::exec::scheduler;
use x86_64;
use x86_64::PrivilegeLevel;
use x86_64::VirtAddr;
use x86_64::registers::{model_specific::EferFlags, rflags::{self, RFlags}};
use x86_64::registers::model_specific::LStar;
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
use raw_cpuid::{CpuId, CpuIdResult};

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
static mut INTERRUPT_STACK: [u8; 0x4000] = [0; 0x4000];
static mut SCHEDULER_INTERRUPT_STACK: [u8; 0x4000] = [0; 0x4000];

pub static mut DO_CONTEXT_SWITCH_NEXT_TIME: bool = false;

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

    pub static ref USER_CS: u64 = (GDT.1.user_code_selector.0 | PrivilegeLevel::Ring3 as u16) as u64;
    pub static ref USER_SS: u64 = (GDT.1.user_data_selector.0 | PrivilegeLevel::Ring3 as u16) as u64;

    pub static ref KERNEL_CS: u64 = (GDT.1.kernel_code_selector.0 | PrivilegeLevel::Ring0 as u16) as u64;
    pub static ref KERNEL_SS: u64 = (GDT.1.kernel_data_selector.0 | PrivilegeLevel::Ring0 as u16) as u64;

    pub static ref USER_DS: SegmentSelector = {
        let mut sel = GDT.1.user_data_selector;
        sel.0 |= PrivilegeLevel::Ring3 as u16;
        sel
    };
}

pub fn init() {
    GDT.0.load();
    
    unsafe {
        segmentation::CS::set_reg(GDT.1.kernel_code_selector);
        tables::load_tss(GDT.1.tss_selector);
        let handler_addr = system_call_trap_handler as *const () as u64;
        asm!("\
        xor rdx, rdx
        mov rax, 0x200
        wrmsr",
        in("rcx") 0xc0000084 as u32);
        LStar::write(VirtAddr::new(handler_addr));
        Star::write(GDT.1.user_code_selector, GDT.1.user_data_selector, GDT.1.kernel_code_selector, GDT.1.kernel_data_selector).expect("GDT_CONFIG_FAILURE");
        Efer::write(Efer::read() | EferFlags::SYSTEM_CALL_EXTENSIONS);
    }

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

    // set up write combining (PAT field PA7)
    let pat_mask: u64 = 1 << 56;
    let pat_antimask: u64 = !(3 << 57);
    let mut pat_msr = registers::model_specific::Msr::new(0x277);
    let pat_val = unsafe { (pat_msr.read() | pat_mask) & pat_antimask };
    unsafe { pat_msr.write(pat_val) };
}

pub fn cpuid() -> CpuId {
    CpuId::with_cpuid_fn(|a, c| {
        let result = unsafe { core::arch::x86_64::__cpuid_count(a, c) };
        CpuIdResult {
            eax: result.eax,
            ebx: result.ebx,
            ecx: result.ecx,
            edx: result.edx,
        }
    })
}

pub fn enable_scheduler() {
    disable_interrupts();
    unsafe {
        IDT[interrupts::HardwareInterrupt::Timer.as_usize()].set_handler_addr(VirtAddr::new(task::timer_handler_save_context as u64)).set_stack_index(SCHEDULER_INTERRUPT_IST_INDEX);
        scheduler::context_switch(None);
    }
}

#[no_mangle]
unsafe extern "C" fn allocate_syscall_stack() -> *mut u8 {
    alloc(Layout::from_size_align_unchecked(0x2000, 0x1000)).offset(0x2000)
}

#[no_mangle]
unsafe extern "C" fn drop_syscall_stack(syscall_stack: *mut u8) {
    dealloc(syscall_stack, Layout::from_size_align_unchecked(0x2000, 0x1000));
}

#[naked]
unsafe extern "C" fn system_call_trap_handler() {
    asm!("
    cli
    push rcx // sysretq rip
    push r11 // sysretq rflags
    push r15 // callee-saved registers
    push r14
    push r13
    push r12
    push rbp
    push rbx
    mov r9, rsp  // save user stack
    push r8      // arg3
    push rdx     // arg2
    push rsi     // arg1
    push rdi     // arg0
    push rax     // syscall
    push r9 // user stack is here
    call allocate_syscall_stack
    pop r9  // user stack is here
    pop rdi      // syscall
    pop rsi      // arg0
    pop rdx      // arg1
    pop rcx      // arg2
    pop r8       // arg3
    mov rsp, rax // set syscall stack
    push r9 // save user stack
    call system_call
    pop r9  // user stack
    sub rsp, 0x2000
    mov rdi, rsp // argument to drop_syscall_stack
    mov rsp, r9  // restore user stack
    push rax // return value
    call drop_syscall_stack
    pop rax
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

unsafe extern "x86-interrupt" fn flag_preempt(_stack_frame: InterruptStackFrame) {
    DO_CONTEXT_SWITCH_NEXT_TIME = true;
    pic::end_of_interrupt(interrupts::HardwareInterrupt::Timer);
}

pub fn atomic_no_preempt<F>(f: F)
where F: FnOnce() {
    unsafe {
        asm!("cli");
        IDT[interrupts::HardwareInterrupt::Timer.as_usize()].set_handler_addr(VirtAddr::new(flag_preempt as u64));
        asm!("sti");
    }
    f();
    unsafe {
        asm!("cli");
        IDT[interrupts::HardwareInterrupt::Timer.as_usize()].set_handler_addr(VirtAddr::new(task::timer_handler_save_context as u64)).set_stack_index(SCHEDULER_INTERRUPT_IST_INDEX);
        asm!("sti");
        while DO_CONTEXT_SWITCH_NEXT_TIME {}
    }
}

pub fn atomic_no_interrupts<F>(f: F)
where F: FnOnce() {
    let flags = rflags::read();
    disable_interrupts();
    f();
    if flags.contains(RFlags::INTERRUPT_FLAG) {
        enable_interrupts();
    }
}

pub fn intflag() -> bool {
    rflags::read().contains(RFlags::INTERRUPT_FLAG)
}

pub fn halt() {
    instructions::hlt();
}

pub fn grinding_halt() -> ! {
    disable_interrupts();
    loop {
        instructions::hlt();
    }
}

#[inline(always)]
pub fn enable_interrupts() {
    instructions::interrupts::enable();
}

#[inline(always)]
pub fn disable_interrupts() {
    unsafe { asm!("cli"); }
}

pub fn register_interrupt_handler(int: interrupts::HardwareInterrupt, handler: extern "x86-interrupt" fn(idt::InterruptStackFrame)) {
    unsafe {
        IDT[int.as_usize()].set_handler_fn(handler).set_stack_index(INTERRUPT_IST_INDEX);
        IDT.load();
    }
}
