use crate::{*, exec::{ExecutableInfo, SectionType}};
use {dev::*, namespace};
use core::arch::asm;
use dev::hal::{cpu, pic, mem::*, interrupts};
use exec::scheduler;
use x86_64::structures::paging::{PageTableFlags, PageTable};
use core::slice;

use super::mem::page_mapper::addr_to_page_table;

const STACK_SIZE: u64 = 0x4000;

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

    pub fn knew(rip: u64, rsp: u64) -> TaskContext {
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
            rsp,
            ss: *cpu::KERNEL_SS,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Process {
    page_table: u64,
    pub threads: Vec<u32>,
}

impl Process {
    pub fn new(page_table: u64) -> Process {
        Process {
            page_table,
            threads: Vec::new(),
        }
    }

    #[inline(always)]
    pub fn die(&self) {
        unsafe {
            enable_page_table(addr_to_page_table(KERNEL_PAGE_TABLE));
        }
        page_mapper::unmap_userspace_page_tables(self.page_table);
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub state: TaskContext,
    pub page_table: u64,
    pub stack_base: u64,
    pub stack_size: u64,
    pub process_id: u32,
    pub zombie: bool,
    pub suspended: bool,
    pub joiner: Option<u32>,
}

impl Task {
    pub fn new(code_addr: u64, stack_top: u64, stack_base: u64, page_table: u64, user_mode: bool, process_id: u32) -> Task {
        Task {
            state: match user_mode {
                true => TaskContext::new(code_addr, stack_top),
                false => TaskContext::knew(code_addr, stack_top),
            },
            page_table,
            stack_base,
            stack_size: stack_top - stack_base,
            zombie: false,
            suspended: false,
            process_id,
            joiner: None,
        }
    }

    #[inline(always)]
    pub fn restore_state(&self) {
        unsafe {
            let val = self.page_table | 0x18;
            asm!("mov cr3, {0}", in(reg) val);
        }
    }

    #[inline(always)]
    pub fn die(&self) {
        let page_table = unsafe { ((self.page_table + PHYSICAL_MEMORY_OFFSET ) as *mut PageTable).as_mut().unwrap() };
        // free up memory used as stack
        for i in (self.stack_base..(self.stack_base + self.stack_size)).step_by(0x1000) {
            if let Some(tran) = page_mapper::translate_addr_using_table(page_table, i as usize) {
                page_mapper::unmap_addr(page_table, i);
                unsafe {
                    FRAME_ALLOCATOR.free_frame(tran);
                }
            }
        }
    }

    pub unsafe fn exec(application: ExecutableInfo, process_id: u32) -> Result<(), Error> {
        asm!("cli");
        let user_page_table = page_mapper::copy_over_kernel_tables_but_not_userspace_ones();
        let user_page_table_phys = (user_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        let flags = Some(PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE);

        if let Some(file) = namespace::get_file_handle(application.file_handle) {
            for section in application.sections { // load all sections and prepare memory
                if section.section_type != SectionType::Load {
                    return Err(Error::InvalidExecutable); // so far we have no dynamic linking and interpreter support
                }

                let virt_base = section.virt_address as u64;
                let mut bytes_left = section.size_in_file;
                file.seek(section.file_offset)?;
                for i in (0..(section.size_in_memory + 0xFFF) / 0x1000).step_by(0x1000) { // map all pages required and load data
                    let frame = FRAME_ALLOCATOR.allocate_frame();
                    page_mapper::map_addr(user_page_table, virt_base + i as u64, frame, flags);
                    serial_println!("Mapping frame {:x} to page {:x}", frame, virt_base + i as u64);
                    let frame_buf = (PHYSICAL_MEMORY_OFFSET + frame) as *mut u8;
                    for j in 0..0x1000 { // clear frame with 0s
                        *frame_buf.offset(j) = 0;
                    }
                    let slice_len = if bytes_left >= 0x1000 {
                        0x1000
                    } else {
                        bytes_left
                    };
                    file.read(slice::from_raw_parts_mut(frame_buf, slice_len))?;
                    if bytes_left >= 0x1000 {
                        bytes_left -= 0x1000;
                    } else {
                        bytes_left = 0;
                    }
                }
            }
            // create process
            scheduler::add_process(Process::new(user_page_table_phys))?;
            // create main thread
            scheduler::add_thread(process_id, Self::exec_thread(application.virt_entry_point as u64, process_id)?);
            Ok(())
        } else {
            Err(Error::EntryNotFound)
        }
    }

    pub unsafe fn exec_thread(entry_point: u64, process_id: u32) -> Result<Task, Error> {
        asm!("cli");
        let user_page_table = ((scheduler::get_process(process_id)?.page_table + PHYSICAL_MEMORY_OFFSET) as *mut PageTable).as_mut().unwrap();
        let user_page_table_phys = (user_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        let flags = Some(PageTableFlags::WRITABLE | PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE);
        // prepare stack
        let mut user_stack_virt_base = 0x60000000;
        // find location for new stack
        while let Some(_) = page_mapper::translate_addr(user_stack_virt_base as usize) {
            user_stack_virt_base += STACK_SIZE;
        }
        for i in 0..(STACK_SIZE / 0x1000) {
            let stack_frame = FRAME_ALLOCATOR.allocate_frame();
            page_mapper::map_addr(user_page_table, user_stack_virt_base + (i * 0x1000), stack_frame, flags);
        }
        asm!("sti");
        Ok(Task::new(entry_point, user_stack_virt_base + STACK_SIZE, user_stack_virt_base, user_page_table_phys, true, process_id))
    }

    pub unsafe fn kexec(application: unsafe fn(), process_id: u32) -> Result<(), Error> {
        asm!("cli");
        let child_page_table = page_mapper::copy_over_kernel_tables_but_not_userspace_ones();
        let child_page_table_phys = (child_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        scheduler::add_process(Process::new(child_page_table_phys));
        scheduler::add_thread(process_id, Self::kexec_thread(application, process_id)?);
        asm!("sti");
        Ok(())
    }

    pub unsafe fn kexec_thread(application: unsafe fn(), process_id: u32) -> Result<Task, Error> {
        asm!("cli");
        let child_page_table = ((scheduler::get_process(process_id)?.page_table + PHYSICAL_MEMORY_OFFSET) as *mut PageTable).as_mut().unwrap();
        let child_page_table_phys = (child_page_table as *const _ as u64) - PHYSICAL_MEMORY_OFFSET;
        let mut child_stack_virt_base = 0x60000000;
        // find location for new stack
        while let Some(_) = page_mapper::translate_addr_using_table(child_page_table, child_stack_virt_base as usize) {
            child_stack_virt_base += STACK_SIZE;
        }
        for i in 0..(STACK_SIZE / 0x1000) {
            let stack_frame = FRAME_ALLOCATOR.allocate_frame();
            page_mapper::map_addr(child_page_table, child_stack_virt_base + (i * 0x1000), stack_frame, None);
        }
        asm!("sti");
        Ok(Task::new(application as u64, (child_stack_virt_base + STACK_SIZE) as u64, child_stack_virt_base, child_page_table_phys, false, process_id))
    }
}

#[naked]

#[naked]
#[no_mangle]
pub unsafe extern "C" fn timer_handler_save_context() {
    asm!("cli; push r15; push r14; push r13; push r12; push r11; push r10; push r9;\
    push r8; push rdi; push rsi; push rdx; push rcx; push rbx; push rax; push rbp;\
    mov rdi, rsp; call timer_handler_context_switch_part_2;", options(noreturn));
}

#[no_mangle]
pub unsafe extern "C" fn timer_handler_context_switch_part_2(context: *const TaskContext) {
    cpu::DO_CONTEXT_SWITCH_NEXT_TIME = false;
    pic::end_of_interrupt(interrupts::HardwareInterrupt::Timer);
    scheduler::context_switch(Some((*context).clone()), true);
}

pub fn trigger_context_switch() {
    cpu::enable_interrupts();
    unsafe {
        asm!("int 0x20"); // trigger timer interrupt
    }
}

#[inline(always)]
pub unsafe fn restore_registers(context: &TaskContext) {
    //serial_println!("New context rip: {:x} rsp: {:x}", context.rip, context.rsp);
    asm!("mov rsp, {0};\
            pop rbp; pop rax; pop rbx; pop rcx; pop rdx; pop rsi; pop rdi; pop r8; pop r9;\
            pop r10; pop r11; pop r12; pop r13; pop r14; pop r15; iretq;", in(reg) context as *const _ as u64);
}
