use crate::*;
use core::arch::asm;
use alloc::alloc::Layout;
use dev::hal::{cpu, pic, mem::*, interrupts};
use task::scheduler::*;
use alloc::vec::Vec;
use x86_64::{PhysAddr, VirtAddr, registers::{control::{Cr3, Cr3Flags}, rflags::RFlags, segmentation::{DS, Segment}}, structures::paging::*};

#[repr(C, align(2))]
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub rbp: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl TaskContext {
    pub fn new(rip: u64, rsp: u64) -> TaskContext {
        TaskContext {
            rbp: 0,
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip,
            cs: *cpu::USER_CS,
            rflags: 0x200,
            rsp,
            ss: *cpu::USER_SS,
        }
    }

    pub fn knew(rip: u64) -> TaskContext {
        TaskContext {
            rbp: 0,
            rax: 0,
            rbx: 0,
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,
            rip,
            cs: *cpu::KERNEL_CS,
            rflags: 0x200,
            rsp: 0,
            ss: *cpu::KERNEL_SS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub state: TaskContext,
    pub page_table: u64,
    _stack_holder: Vec<u8>,
}

impl Task {
    pub fn new(code_addr: u64, stack_addr: u64, page_table: u64, _stack_holder: Vec<u8>) -> Task {
        Task {
            state: TaskContext::new(code_addr, stack_addr),
            page_table,
            _stack_holder,
        }
    }

    pub fn new_stack(code_addr: u64, stack_size: usize) -> Task {
        let mut task = Task {
            state: TaskContext::knew(code_addr),
            page_table: Cr3::read().0.start_address().as_u64(),
            _stack_holder: Vec::<u8>::with_capacity(stack_size),
        };
        task.state.rsp = &task._stack_holder as *const _ as u64;
        task
    }

    pub fn save_state(&mut self) {
        self.page_table = Cr3::read().0.start_address().as_u64();
    }

    #[inline(always)]
    pub fn restore_state(&self) {
        unsafe {
            Cr3::write(PhysFrame::from_start_address_unchecked(PhysAddr::new(self.page_table)), Cr3Flags::all());
            //DS::set_reg(*cpu::USER_DS);
        }
    }

    pub unsafe fn exec(application: unsafe extern "C" fn()) {
        let phys_mem_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.try_into().unwrap());
        let kernel_page_table = active_level_4_table(phys_mem_offset);
        let user_page_table = page_mapper::new_l4_table();
        for (i, ent) in kernel_page_table.iter().enumerate() {
            if !ent.is_unused() {
                user_page_table[i] = ent.clone();
            }
        }
        let kernel_page_table_phys = (kernel_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        let user_page_table_phys = (user_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        enable_page_table(user_page_table);
        let flags = Some(PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::GLOBAL);
        let user_virt_base = 0x40000000;
        let user_phys_base = page_mapper::translate_addr(application as usize).unwrap();
        for i in 0..100 {
            let off = i * 0x1000;
            page_mapper::map_addr(user_virt_base + off, user_phys_base + off, flags).expect("peace");
        }
        let user_stack = Vec::<u8>::with_capacity(1000);
        let user_stack_virt_base = 0x60000000;
        let user_stack_phys_base = page_mapper::translate_addr(user_stack.as_ptr() as usize).unwrap();
        page_mapper::map_addr(user_stack_virt_base, user_stack_phys_base, flags).expect("fuck you");
        page_mapper::map_addr(user_stack_virt_base + 0x1000, user_stack_phys_base + 0x1000, flags).expect("capybara");
        let user_entry_point_offset = user_phys_base % 0x1000;
        let user_stack_offset = user_stack_phys_base % 0x1000;
        Cr3::write(PhysFrame::from_start_address_unchecked(PhysAddr::new(kernel_page_table_phys)), Cr3Flags::all());
        Scheduler::add_process(Task::new((user_virt_base + user_entry_point_offset + 1) as u64, (user_stack_virt_base + user_stack_offset + 0x1000) as u64, user_page_table_phys as u64, user_stack));
    }

    pub fn kexec(application: unsafe fn()) {
        Scheduler::add_process(Task::new_stack(application as u64, 0x1000));
    }
}

#[naked]
#[no_mangle]
pub unsafe fn timer_handler_save_context() {
    asm!("cli; push r15; push r14; push r13; push r12; push r11; push r10; push r9;\
    push r8; push rdi; push rsi; push rdx; push rcx; push rbx; push rax; push rbp;\
    mov rdi, rsp; call timer_handler_scheduler_part_2;", options(noreturn));
}

#[no_mangle]
pub unsafe extern "C" fn timer_handler_scheduler_part_2(context: *const TaskContext) {
    cpu::DO_CONTEXT_SWITCH_NEXT_TIME = false;
    pic::end_of_interrupt(interrupts::HardwareInterrupt::Timer);
    Scheduler::context_switch(Some((*context).clone()));
}

#[inline(always)]
pub unsafe fn restore_registers(context: &TaskContext) {
    asm!("mov rsp, {0};\
            pop rbp; pop rax; pop rbx; pop rcx; pop rdx; pop rsi; pop rdi; pop r8; pop r9;\
            pop r10; pop r11; pop r12; pop r13; pop r14; pop r15; iretq;", in(reg) context as *const _ as u64);
}
