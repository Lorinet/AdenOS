#![allow(unaligned_references)]

use x86_64::structures::paging::PageTableFlags;

use crate::{*, dev::hal::{mem::{self, page_mapper}, pic}};
use core::arch::asm;

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_ENABLE: u32 = 0b100000000000;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct LAPICRegisters {
    _reserved_0: [u64; 4],
    lapic_id: u32,
    _reserved_1: [u32; 3],
    lapic_version: u32,
    _reserved_2: [u32; 19],
    task_priority: u32,
    _reserved_3: [u32; 3],
    arbitration_priority: u32,
    _reserved_4: [u32; 3],
    processor_priority: u32,
    _reserved_5: [u32; 3],
    eoi_register: u32,
    _reserved_6: [u32; 3],
    remote_read: u32,
    _reserved_7: [u32; 3],
    logical_destination: u32,
    _reserved_8: [u32; 3],
    destination_format: u32,
    _reserved_9: [u32; 3],
    spurious_interrupt_vector: u32,
    _reserved_10: [u32; 3],
    in_service_register: [u32; 32],
    trigger_mode_register: [u32; 32],
    interrupt_request: [u32; 32],
    error_status: u32,
    _reserved_11: [u32; 27],
    lvt_corrected_machine_check_interrupt: u32,
    _reserved_12: [u32; 3],
    interrupt_command: [u32; 5],
    _reserved_13: [u32; 3],
    lvt_timer: u32,
    _reserved_14: [u32; 3],
    lvt_thermal_sensor: u32,
    _reserved_15: [u32; 3],
    lvt_performance_monitoring_counters: u32,
    _reserved_16: [u32; 3],
    lvt_lint0: u32,
    _reserved_17: [u32; 3],
    lvt_lint1: u32,
    _reserved_18: [u32; 3],
    lvt_error: u32,
    _reserved_19: [u32; 3],
    timer_initial_count: u32,
    _reserved_20: [u32; 3],
    timer_current_count: u32,
    _reserved_21: [u32; 19],
    timer_divide_configuration: u32,
    _reserved_22: [u32; 7],
}

#[derive(Debug)]
pub struct LAPIC {
    registers: *mut LAPICRegisters,
    disable_pic_on_init: bool,
}

impl LAPIC {
    pub fn new(base_address: u32, disable_pic_on_init: bool) -> LAPIC {
        LAPIC {
            registers: unsafe { (base_address as u64 + mem::PHYSICAL_MEMORY_OFFSET) as *mut LAPICRegisters },
            disable_pic_on_init
        }
    }
}

impl dev::Device for LAPIC {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        if self.disable_pic_on_init {
            pic::deinit();
        }
        let mut rax: u32;
        let mut rdx: u32;
        unsafe {
            let phys_addr = self.registers as u64 - mem::PHYSICAL_MEMORY_OFFSET;
            let virt_addr = 0x90000000;
            serial_println!("peace");
            page_mapper::map_addr(page_mapper::get_l4_table(), virt_addr, phys_addr, Some(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE));
            serial_println!("pizza");
            self.registers = virt_addr as *mut LAPICRegisters;
            serial_println!("capybara");
            asm!("rdmsr", in("rcx") IA32_APIC_BASE_MSR, out("rax") rax, out("rdx") rdx);
            rax |= IA32_APIC_BASE_MSR_ENABLE;
            asm!("wrmsr", in("rcx") IA32_APIC_BASE_MSR, in("rax") rax, in("rdx") rdx);
            (*self.registers).task_priority = 0x20;
            (*self.registers).lvt_timer = 0x10000;
            (*self.registers).lvt_performance_monitoring_counters = 0x10000;
            (*self.registers).lvt_error = 0x10000;
            (*self.registers).lvt_lint0 = 0x8700;
            (*self.registers).lvt_lint1 = 0x400;
            (*self.registers).spurious_interrupt_vector = 0x10;
            (*self.registers).lvt_lint0 = 0x8700;
            (*self.registers).lvt_lint1 = 0x400;
            serial_println!("{:#x?}", *self.registers);
        }
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }

    fn device_name(&self) -> &str {
        "System/LAPIC"
    }
}