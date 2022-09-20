#![allow(unaligned_references)]

use crate::{*, dev::hal::{mem, pic}};
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
        serial_println!("Here it gets fucked");
        let mut rax: u32;
        let mut rdx: u32;
        unsafe {
            asm!("rdmsr", in("rcx") IA32_APIC_BASE_MSR, out("rax") rax, out("rdx") rdx);
            asm!("wrmsr", in("rcx") IA32_APIC_BASE_MSR, in("rax") rax, in("rdx") (rdx | IA32_APIC_BASE_MSR_ENABLE));
            //asm!("rdmsr", in("rcx") IA32_APIC_BASE_MSR, out("rax") high, out("rdx") low);
        }
        let apic_msr: u64 = ((rdx as u64) << 32) | rax as u64;
        serial_println!("{:#b}", apic_msr);
        serial_println!("{:#x}", apic_msr);
        unsafe {
            (*self.registers).spurious_interrupt_vector = 0x100;
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