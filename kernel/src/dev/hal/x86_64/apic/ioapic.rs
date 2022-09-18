#![allow(unaligned_references)]

use crate::{*, dev::hal::{mem, cpu}};
use alloc::{string::*, vec};
use modular_bitfield::{*, specifiers::*};
use alloc::vec::*;
use super::madt::MADTEntryIOAPICInterruptSourceOverride;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum TriggerMode {
    ConformsToSpecs = 0,
    Edge = 1,
    Level = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum APICTriggerMode {
    Edge = 0,
    Level = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum Polarity {
    ConformsToSpecs = 0,
    ActiveHigh = 1,
    ActiveLow = 3,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum APICPolarity {
    ActiveHigh = 0,
    ActiveLow = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum DestinationMode {
    Physical = 0,
    Logical = 1,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum DeliveryMode {
    Fixed = 0,
    LowestPriority = 1,
    SMI = 2,
    NMI = 4,
    Init = 5,
    ExtInt = 7,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct IOAPICRedirectionTableEntry {
    pub vector: u8,
    pub delivery_mode: B3,
    pub destination_mode: bool,
    pub delivery_status: bool,
    pub pin_polarity: bool,
    pub remote_irr: bool,
    pub trigger_mode: bool,
    pub mask: bool,
    _reserved: B39,
    pub destination: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
union IOAPICRedirectionTableEntryUnion {
    entry: IOAPICRedirectionTableEntry,
    data: (u32, u32),
}

#[derive(Debug)]
pub struct IOAPIC {
    io_register_select: *mut u32,
    io_register_window: *mut u32,
    pub ioapic_id: u8,
    pub redirection_entries: u8,
    pub version: u8,
    pub interrupt_base: u32,
    pub interrupt_source_overrides: Vec<MADTEntryIOAPICInterruptSourceOverride>,
    name: String,
}

impl IOAPIC {
    pub fn new(base_address: u32, ioapic_id: u8, interrupt_base: u32) -> IOAPIC {
        IOAPIC {
            io_register_select: unsafe { (base_address as u64 + mem::PHYSICAL_MEMORY_OFFSET) as *mut u32 },
            io_register_window: unsafe { (base_address as u64 + 0x10 + mem::PHYSICAL_MEMORY_OFFSET) as *mut u32 },
            ioapic_id,
            redirection_entries: 0,
            version: 0,
            interrupt_base,
            interrupt_source_overrides: vec![],
            name: String::from("System/IOAPIC") + &ioapic_id.to_string(),
        }
    }

    pub fn read_register(&self, register: u32) -> u32 {
        unsafe {
            *self.io_register_select = register;
            *self.io_register_window
        }
    }

    pub fn write_register(&self, register: u32, value: u32) {
        unsafe {
            *self.io_register_select = register;
            *self.io_register_window = value;
        }
    }

    pub fn read_redirection_table_entry(&self, index: u32) -> IOAPICRedirectionTableEntry {
        let upper = self.read_register(0x10 + (2 * index));
        let lower = self.read_register(0x10 + (2 * index) + 1);
        unsafe {
            let un = IOAPICRedirectionTableEntryUnion {
                data: (upper, lower),
            };
            un.entry
        }
    }

    pub fn write_redirection_table_entry(&self, index: u32, entry: IOAPICRedirectionTableEntry) {
        let upper: u32;
        let lower: u32;
        unsafe {
            let un = IOAPICRedirectionTableEntryUnion {
                entry,
            };
            (upper, lower) = un.data; 
        }
        self.write_register(0x10 + (2 * index), upper);
        self.write_register(0x10 + (2 * index + 1), lower);
    }
}

impl dev::Device for IOAPIC {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        self.version = self.read_register(0x01) as u8;
        self.redirection_entries = ((self.read_register(0x01) >> 16) + 1) as u8;
        serial_println!("IOAPIC Version: {:#x}", self.version);
        serial_println!("IOAPIC Redirection entries: {}", self.redirection_entries);

        let bsp_lapic_id = cpu::cpuid().get_feature_info().unwrap().initial_local_apic_id();
        for over in &self.interrupt_source_overrides {
            let entry = IOAPICRedirectionTableEntry::new()
            .with_destination(bsp_lapic_id)
            .with_destination_mode(DestinationMode::Physical as u8 == 1)
            .with_delivery_mode(DeliveryMode::Fixed as u8)
            .with_mask(false)
            .with_pin_polarity(match Polarity::from_u8(over.flags.polarity()).unwrap() {
                Polarity::ConformsToSpecs => APICPolarity::ActiveHigh,
                Polarity::ActiveHigh => APICPolarity::ActiveHigh,
                Polarity::ActiveLow => APICPolarity::ActiveLow,
            } as u8 == 1)
            .with_trigger_mode(match TriggerMode::from_u8(over.flags.trigger_mode()).unwrap() {
                TriggerMode::ConformsToSpecs => APICTriggerMode::Edge,
                TriggerMode::Edge => APICTriggerMode::Edge,
                TriggerMode::Level => APICTriggerMode::Level,
            } as u8 == 1)
            .with_vector(32 + match over.irq_source {
                2 => 0,
                irq => irq,
            });
            let redir_table_index = over.global_system_interrupt - self.interrupt_base;
            serial_println!("Redir int {}", redir_table_index);
            self.write_redirection_table_entry(redir_table_index, entry)
        }
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }

    fn device_name(&self) -> &str {
        self.name.as_str()
    }
}