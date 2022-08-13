use crate::*;
use core::arch::asm;
use alloc::alloc::Layout;
use dev::hal::{cpu, pic, mem::*, interrupts};
use task::scheduler::*;
use alloc::vec::Vec;
use x86_64::{PhysAddr, VirtAddr, registers::control::{Cr3, Cr3Flags}, structures::paging::*};

#[repr(C)]
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

#[derive(Debug, Clone)]
pub enum TaskState {
    Start(usize, usize), // rip, rsp
    Running(TaskContext),
}

pub struct Task {
    pub state: TaskState,
    pub page_table: u64,
    _stack_holder: Vec<u8>,
}

impl Task {
    pub fn new(code_addr: usize, stack_addr: usize, page_table: u64, _stack_holder: Vec<u8>) -> Task {
        Task {
            state: TaskState::Start(code_addr, stack_addr),
            page_table,
            _stack_holder,
        }
    }

    pub fn save_state(&mut self, context: *const TaskContext) {
        unsafe { self.state = TaskState::Running((*context).clone()); }
        if let TaskState::Running(ctx) = &self.state {
        }
        self.page_table = Cr3::read().0.start_address().as_u64();
    }

    #[inline(always)]
    pub fn restore_state(&self) {
        match &self.state {
            TaskState::Running(ctx) => {
                unsafe {
                    Cr3::write(PhysFrame::from_start_address_unchecked(PhysAddr::new(self.page_table)), Cr3Flags::all());
                    let cl = ctx.clone();
                    println!("RESTORE: {:x}");
                    asm!("mov rsp, {0};\
                    pop rbp; pop rax; pop rbx; pop rcx; pop rdx; pop rsi; pop rdi; pop r8; pop r9;\
                    pop r10; pop r11; pop r12; pop r13; pop r14; pop r15; iretq;", in(reg) &cl as *const _ as u64);
                }
            }
            TaskState::Start(rip, rsp) => {
                unsafe {
                    Cr3::write(PhysFrame::from_start_address_unchecked(PhysAddr::new(self.page_table)), Cr3Flags::all());
                    cpu::enter_user_mode(*rip, *rsp);
                }
            }
        }
    }

    pub unsafe fn exec(application: unsafe fn()) {
        let phys_mem_offset = VirtAddr::new(PHYSICAL_MEMORY_OFFSET.try_into().unwrap());
        let kernel_page_table = active_level_4_table(phys_mem_offset);
        let user_page_table = page_mapper::new_l4_table();
        for (i, ent) in kernel_page_table.iter().enumerate() {
            if !ent.is_unused() {
                user_page_table[i] = ent.clone();
            }
        }
        let kernel_page_table_phys = (kernel_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        let user_page_table_phys = page_mapper::translate_addr(user_page_table as *const _ as usize).unwrap();
        enable_page_table(user_page_table);
        let flags = Some(PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::GLOBAL);
        let user_virt_base = 0x40000000000;
        let user_phys_base = page_mapper::translate_addr(application as usize).unwrap();
        page_mapper::map_addr(user_virt_base, user_phys_base, flags).expect("peace");
        page_mapper::map_addr(user_virt_base + 0x1000, user_phys_base + 0x1000, flags).expect("fuck you");
        let user_stack = Vec::<u8>::with_capacity(1000);
        let user_stack_virt_base = 0x60000000000;
        let user_stack_phys_base = page_mapper::translate_addr(user_stack.as_ptr() as usize).unwrap();
        page_mapper::map_addr(user_stack_virt_base, user_stack_phys_base, flags).expect("donuts");
        page_mapper::map_addr(user_stack_virt_base + 0x1000, user_stack_phys_base + 0x1000, flags).expect("capybara");
        let user_entry_point_offset = user_phys_base % 0x1000;
        let user_stack_offset = user_stack_phys_base % 0x1000;
        Cr3::write(PhysFrame::from_start_address_unchecked(PhysAddr::new(kernel_page_table_phys)), Cr3Flags::all());
        SCHEDULER.add_process(Task::new((user_virt_base + user_entry_point_offset + 1) as usize, user_stack_virt_base + user_stack_offset + 0x1000, user_page_table_phys as u64, user_stack));
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
    pic::end_of_interrupt(interrupts::HardwareInterrupt::Timer);
    SCHEDULER.context_switch(Some(context));
}