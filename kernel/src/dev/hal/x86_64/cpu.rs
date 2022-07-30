use crate::*;
use dev;
use x86_64;
use x86_64::structures::idt;
use x86_64::structures::tss;
use x86_64::structures::gdt;
use x86_64::instructions;
use x86_64::instructions::tables;
use x86_64::instructions::segmentation;
use x86_64::instructions::segmentation::Segment;
use lazy_static::lazy_static;
use super::*;

const DOUBLE_FAULT_IST_INDEX: u16 = 0;

struct Selectors {
    code_selector: gdt::SegmentSelector,
    tss_selector: gdt::SegmentSelector,
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
        let code_selector = gdt.add_entry(gdt::Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(gdt::Descriptor::tss_segment(&TSS));
        (gdt, Selectors { code_selector, tss_selector })
    };
}

pub fn init() {
    GDT.0.load();
    unsafe {
        segmentation::CS::set_reg(GDT.1.code_selector);
        tables::load_tss(GDT.1.tss_selector);
    }
    unsafe {
        IDT.breakpoint.set_handler_fn(interrupts::breakpoint::breakpoint_handler);
        IDT.double_fault.set_handler_fn(interrupts::double_fault::double_fault_handler).set_stack_index(DOUBLE_FAULT_IST_INDEX);
        IDT[HardwareInterrupt::Timer.as_usize()].set_handler_fn(interrupts::timer::timer_handler);
        IDT.load();
    }
    pic::init();
    pic::enable_interrupts();
}

pub fn trigger_breakpoint() {
    instructions::interrupts::int3();
}

pub fn atomic_no_interrupts<F>(f: F)
where F: FnOnce() {
    pic::disable_interrupts();
    f();
    pic::enable_interrupts();
}

pub fn halt() {
    instructions::hlt();
}

pub fn grinding_halt() -> ! {
    pic::disable_interrupts();
    loop {
        instructions::hlt();
    }
}

pub fn pic_end_of_interrupt(int: HardwareInterrupt) {
    pic::end_of_interrupt(int);
}

pub fn register_interrupt_handler(int: HardwareInterrupt, handler: extern "x86-interrupt" fn(idt::InterruptStackFrame)) {
    unsafe {
        IDT[int.as_usize()].set_handler_fn(handler);
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum HardwareInterrupt {
    Timer = pic::PIC_1_OFFSET,
    Keyboard,
}

impl HardwareInterrupt {
    pub fn as_u8(self) -> u8 {
        self as u8
    }

    pub fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}