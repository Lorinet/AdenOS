#![allow(unaligned_references)]

use alloc::{string::String, vec, vec::Vec};
use x86_64::structures::paging::PageTableFlags;
use modular_bitfield::{bitfield, specifiers::*};

use crate::{*, dev::hal::{mem::{self, page_mapper}}, namespace::Resource};
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
    error_status: LAPICError,
    _reserved_11: [u32; 27],
    lvt_corrected_machine_check_interrupt: u32,
    _reserved_12: [u32; 3],
    interrupt_command_low: u32,
    _reserved_13: [u32; 3],
    interrupt_command_high: u32,
    _reserved_14: [u32; 3],
    lvt_timer: u32,
    _reserved_15: [u32; 3],
    lvt_thermal_sensor: u32,
    _reserved_16: [u32; 3],
    lvt_performance_monitoring_counters: u32,
    _reserved_17: [u32; 3],
    lvt_lint0: u32,
    _reserved_18: [u32; 3],
    lvt_lint1: u32,
    _reserved_19: [u32; 3],
    lvt_error: u32,
    _reserved_20: [u32; 3],
    timer_initial_count: u32,
    _reserved_21: [u32; 3],
    timer_current_count: u32,
    _reserved_22: [u32; 19],
    timer_divide_configuration: u32,
    _reserved_23: [u32; 7],
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum TriggerMode {
    Edge = 0,
    Level = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum DestinationMode {
    Physical = 0,
    Logical = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum DeliveryMode {
    Fixed = 0,
    LowestPriority = 1,
    SMI = 2,
    NMI = 4,
    Init = 5,
    StartUp = 6,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
enum DestinationShorthand {
    None = 0,
    OnlySelf = 1,
    AllIncludingSelf = 2,
    AllExcludingSelf = 3,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct LAPICInterrupt {
    pub vector: u8,
    pub delivery_mode: B3,
    pub destination_mode: bool,
    pub delivery_status: bool,
    _reserved_0: bool,
    pub level: bool,
    pub trigger_mode: bool,
    _reserved_1: B2,
    pub destination_shorthand: B2,
    _reserved_2: B12,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
union LAPICInterruptUnion {
    int: LAPICInterrupt,
    data: (u32, u32),
}

#[bitfield]
#[repr(C, packed, u32)]
#[derive(Copy, Clone, Debug)]
pub struct LAPICError {
    pub send_checksum_error: bool,
    pub receive_checksum_error: bool,
    pub send_accept_error: bool,
    pub receive_access_error: bool,
    pub redirectable_ipi: bool,
    pub send_illegal_vector: bool,
    pub received_illegal_vector: bool,
    pub illegal_register_address: bool,
    _reserved: B24,
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

    fn send_interrupt(&mut self, int: LAPICInterrupt) {
        let un = LAPICInterruptUnion {
            int,
        };
        unsafe {
            let (low, high) = un.data;
            (*self.registers).interrupt_command_high = high;
            (*self.registers).interrupt_command_low = low;
        }
    }
}

impl dev::Device for LAPIC {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        return Ok(());
        if self.disable_pic_on_init {
            //pic::deinit();
        }
        let mut rax: u32;
        let mut rdx: u32;
        unsafe {
            let phys_addr = self.registers as u64 - mem::PHYSICAL_MEMORY_OFFSET;
            let virt_addr = 0x90000000;
            // map to strong UC memory
            page_mapper::map_addr(page_mapper::get_l4_table(), virt_addr, phys_addr, Some(PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE));
            self.registers = virt_addr as *mut LAPICRegisters;
            asm!("rdmsr", in("rcx") IA32_APIC_BASE_MSR, out("rax") rax, out("rdx") rdx);
            rax |= IA32_APIC_BASE_MSR_ENABLE;
            asm!("wrmsr", in("rcx") IA32_APIC_BASE_MSR, in("rax") rax, in("rdx") rdx);
            (*self.registers).task_priority = 0;
            (*self.registers).spurious_interrupt_vector = 0b00000000000000000000000111111111;
        }
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("System"), String::from("LAPIC")]
    }
}