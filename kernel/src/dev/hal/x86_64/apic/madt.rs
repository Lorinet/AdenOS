use modular_bitfield::{*, specifiers::*};

use crate::dev::hal::acpi::tables::ACPITable;
use core::mem::size_of;

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTTable {
    pub acpi_header: ACPITable,
    pub lapic_address: u32,
    pub flags: u32,
}

impl MADTTable {
    pub fn iter(&self) -> MADTIterator {
        self.into_iter()
    }
}

impl From<&'static ACPITable> for &MADTTable {
    fn from(table: &'static ACPITable) -> Self {
        unsafe { (table as *const ACPITable as *const MADTTable).as_ref().unwrap() }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum MADTEntryType {
    LAPIC = 0,
    IOAPIC = 1,
    IOAPICInterruptSourceOverride = 2,
    IOAPICNMISource = 3,
    LAPICNonMaskableInterrupts = 4,
    LAPICAddressOverride = 5,
    X2LAPIC = 9,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryHeader {
    pub entry_type: MADTEntryType,
    pub record_length: u8,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryLAPIC {
    pub header: MADTEntryHeader,
    pub acpi_processor_id: u8,
    pub apic_id: u8,
    pub flags: u32,
}

impl From<&'static MADTEntryHeader> for &MADTEntryLAPIC {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryLAPIC).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryIOAPIC {
    pub header: MADTEntryHeader,
    pub ioapic_id: u8,
    _reserved: u8,
    pub ioapic_address: u32,
    pub global_system_interrupt_base: u32,
}

impl From<&'static MADTEntryHeader> for &MADTEntryIOAPIC {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryIOAPIC).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryIOAPICInterruptSourceOverride {
    pub header: MADTEntryHeader,
    pub bus_source: u8,
    pub irq_source: u8,
    pub global_system_interrupt: u32,
    pub flags: IOAPICInterruptSourceFlags,
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct IOAPICInterruptSourceFlags {
    pub polarity: B2,
    pub trigger_mode: B2,
    _reserved: B12,
}

impl From<&'static MADTEntryHeader> for &MADTEntryIOAPICInterruptSourceOverride {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryIOAPICInterruptSourceOverride).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryIOAPICNMISource {
    pub header: MADTEntryHeader,
    pub nmi_source: u8,
    _reserved: u8,
    pub flags: u16,
    pub global_system_interrupt: u32,
}

impl From<&'static MADTEntryHeader> for &MADTEntryIOAPICNMISource {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryIOAPICNMISource).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryLAPICNonMaskableInterrupts {
    pub header: MADTEntryHeader,
    pub acpi_processor_id: u8,
    pub flags: u16,
    pub lint: u8,
}

impl From<&'static MADTEntryHeader> for &MADTEntryLAPICNonMaskableInterrupts {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryLAPICNonMaskableInterrupts).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryLAPICAddressOverride {
    pub header: MADTEntryHeader,
    _reserved: u16,
    pub lapic_base_address: u64,
}

impl From<&'static MADTEntryHeader> for &MADTEntryLAPICAddressOverride {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryLAPICAddressOverride).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MADTEntryX2LAPIC {
    pub header: MADTEntryHeader,
    _reserved: u16,
    pub x2lapic_processor_id: u32,
    pub flags: u32,
    pub acpi_id: u32,
}

impl From<&'static MADTEntryHeader> for &MADTEntryX2LAPIC {
    fn from(header: &'static MADTEntryHeader) -> Self {
        unsafe { (header as *const MADTEntryHeader as *const MADTEntryX2LAPIC).as_ref().unwrap() }
    }
}

impl IntoIterator for &MADTTable {
    type Item = &'static MADTEntryHeader;
    type IntoIter = MADTIterator;

    fn into_iter(self) -> Self::IntoIter {
        let address = (self as *const MADTTable as u64) + (size_of::<MADTTable>() as u64);
        let end_address = (self as *const MADTTable as u64) + self.acpi_header.length as u64;
        MADTIterator {
            address,
            end_address,
        }
    }
}

pub struct MADTIterator {
    address: u64,
    end_address: u64,
}

impl Iterator for MADTIterator {
    type Item = &'static MADTEntryHeader;
    fn next(&mut self) -> Option<Self::Item> {
        if self.address >= self.end_address {
            None
        } else {
            let ent = unsafe { (self.address as *const MADTEntryHeader).as_ref().unwrap() };
            self.address += ent.record_length as u64;
            Some(ent)
        }
    }
}