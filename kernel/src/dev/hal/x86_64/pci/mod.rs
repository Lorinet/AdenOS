use crate::*;
use core::{iter::Iterator, mem::size_of};
use dev::hal::{mem, acpi::tables::*};

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PCIDeviceConfiguration {
    base_address: u64,
    segment_group: u64,
    start_bus: u8,
    end_bus: u8,
    _reserved: u32,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MCFGTable {
    acpi_header: ACPITable,
    _reserved: u64,
}

impl MCFGTable {
    pub fn from_address(phys_address: u64) -> &'static ACPITable {
        unsafe { ((phys_address + mem::PHYSICAL_MEMORY_OFFSET) as *const ACPITable).as_ref().unwrap() }
    }

    pub fn entry_count(&self) -> usize {
        self.acpi_header.entry_count()
    }

    pub fn iter(&self) -> MCFGIterator {
        self.into_iter()
    }
}

impl IntoIterator for &MCFGTable {
    type Item = &'static PCIDeviceConfiguration;
    type IntoIter = MCFGIterator;

    fn into_iter(self) -> Self::IntoIter {
        let address = (self as *const MCFGTable as u64) + (size_of::<MCFGTable>() as u64) + unsafe { mem::PHYSICAL_MEMORY_OFFSET };
        serial_println!("MCFG addr: {:x}", address);
        MCFGIterator {
            address,
            end_address: address + (self.entry_count() * 8) as u64,
        }
    }
}

impl From<&'static ACPITable> for &MCFGTable {
    fn from(table: &'static ACPITable) -> Self {
        unsafe { (table as *const ACPITable as *const Self).as_ref().unwrap() }
    }
}

pub struct MCFGIterator {
    address: u64,
    end_address: u64,
}

impl Iterator for MCFGIterator {
    type Item = &'static PCIDeviceConfiguration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.address >= self.end_address {
            None
        } else {
            let conf = unsafe { (self.address as *const PCIDeviceConfiguration).as_ref().unwrap() };
            self.address += size_of::<PCIDeviceConfiguration>() as u64;
            Some(conf)
        }
    }
}
