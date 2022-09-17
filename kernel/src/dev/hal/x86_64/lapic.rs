#![allow(unaligned_references)]

use crate::{*, dev::hal::mem};
use core::arch::asm;
use super::pic;

const IA32_APIC_BASE_MSR: u32 = 0x1B;
const IA32_APIC_BASE_MSR_ENABLE: u32 = 0x800;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct APICRegisters {
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

pub struct LAPIC {
    registers: * mut APICRegisters,
}

impl LAPIC {
    pub fn new() -> LAPIC {
        LAPIC {
            registers: 0 as *mut APICRegisters,
        }
    }
}

impl dev::Device for LAPIC {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        pic::deinit();
        let high: u32;
        let low: u32;
        unsafe {
            asm!("mov rcx, r10
            rdmsr
            mov r11, rbx", in("r10") IA32_APIC_BASE_MSR, out("rax") high, out("r11") low);
        }
        let apic_base_address: u64 = ((high & 0xfffff000) as u64) | ((low as u64 & 0x0f) << 32);
        let high = ((apic_base_address & 0xfffff0000) as u32) | IA32_APIC_BASE_MSR_ENABLE;
        let low = ((apic_base_address >> 32) & 0x0f) as u32;
        unsafe {
            asm!("wrmsr", in("ecx") IA32_APIC_BASE_MSR, in("rax") high, in("rdx") low);
            self.registers = (apic_base_address + mem::PHYSICAL_MEMORY_OFFSET) as *mut APICRegisters;
            (*self.registers).task_priority = 0;
            (*self.registers).task_priority = 0;
            (*self.registers).spurious_interrupt_vector = 0x100 | 0xff;
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