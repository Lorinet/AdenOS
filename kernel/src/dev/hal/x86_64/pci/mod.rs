use crate::*;
use core::{iter::Iterator, mem::size_of};
use dev::hal::{mem, acpi::tables::*};

pub mod id;

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PCIDeviceHeader {
    pub vendor_id: u16,
    pub device_id: u16,
    pub command: u16,
    pub status: u16,
    pub revision_id: u8,
    pub prog_if: u8,
    pub subclass: u8,
    pub class: u8,
    pub cache_line_size: u8,
    pub latency_timer: u8,
    pub header_type: u8,
    pub bist: u8,
}

impl PCIDeviceHeader {
    pub fn from_address(address: u64) -> &'static PCIDeviceHeader {
        unsafe { (address as *const PCIDeviceHeader).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct PCIHeaderType0 {
    pub pci_device_header: PCIDeviceHeader,
    pub bar_0: u32,
    pub bar_1: u32,
    pub bar_2: u32,
    pub bar_3: u32,
    pub bar_4: u32,
    pub bar_5: u32,
    pub cardbus_cis_pointer: u32,
    pub subsystem_vendor_id: u16,
    pub subsystem_id: u16,
    pub expansion_rom_base_address: u32,
    pub capabilities_pointer: u8,
    _reserved_0: u8,
    _reserved_1: u16,
    _reserved_2: u32,
    pub interrupt_line: u8,
    pub interrupt_pin: u8,
    pub min_grant: u8,
    pub max_latency: u8,
}

impl From<&'static PCIDeviceHeader> for &PCIHeaderType0 {
    fn from(header: &'static PCIDeviceHeader) -> Self {
        unsafe { ((header as *const PCIDeviceHeader) as *const PCIHeaderType0).as_ref().unwrap() }
    }
}

#[derive(Debug)]
pub struct PCIFunction {
    function_address: u64,
}

impl PCIFunction {
    pub fn device_header(&self) -> &'static PCIDeviceHeader {
        unsafe { (self.function_address as *const PCIDeviceHeader).as_ref().unwrap() }
    }
}

#[derive(Debug)]
pub struct PCIDeviceIterator {
    device_address: u64,
    current_function: u64,
}

impl Iterator for PCIDeviceIterator {
    type Item = PCIFunction;
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_function < 8 {
            let function_address = self.device_address + (self.current_function << 12);
            let head = PCIDeviceHeader::from_address(function_address);
            if head.device_id == 0 || head.device_id == 0xffff {
                self.current_function += 1;
            } else {
                let retv = Some(PCIFunction {
                    function_address,
                });
                self.current_function += 1;
                return retv;
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct PCIBusIterator {
    bus_address: u64,
    current_device: u64,
}

impl Iterator for PCIBusIterator {
    type Item = PCIDeviceIterator;
    fn next(&mut self) -> Option<Self::Item> {
        while self.current_device < 32 {
            let device_address = self.bus_address + (self.current_device << 15);
            let head = PCIDeviceHeader::from_address(device_address);
            if head.device_id == 0 || head.device_id == 0xffff {
                self.current_device += 1;
            } else {
                let retv = Some(PCIDeviceIterator {
                    device_address,
                    current_function: 0,
                });
                self.current_device += 1;
                return retv;
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct PCIConfigurationIterator {
    base_address: u64,
    end_bus: u8,
    current_bus: u8,
}

impl Iterator for PCIConfigurationIterator {
    type Item = PCIBusIterator;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_bus < self.end_bus {
            let retv = Some(PCIBusIterator {
                bus_address: self.base_address + ((self.current_bus as u64) << 20),
                current_device: 0,
            });
            self.current_bus += 1;
            return retv;
        }
        None
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct PCIDeviceConfiguration {
    pub base_address: u64,
    pub segment_group: u16,
    pub start_bus: u8,
    pub end_bus: u8,
    _reserved: u32,
}

impl IntoIterator for &PCIDeviceConfiguration {
    type Item = PCIBusIterator;
    type IntoIter = PCIConfigurationIterator;
    fn into_iter(self) -> Self::IntoIter {
        PCIConfigurationIterator {
            base_address: unsafe { self.base_address + mem::PHYSICAL_MEMORY_OFFSET },
            current_bus: self.start_bus,
            end_bus: self.end_bus,
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct MCFGTable {
    pub acpi_header: ACPITable,
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
        let address = (self as *const MCFGTable as u64) + (size_of::<MCFGTable>() as u64);
        let entries = (self.acpi_header.length as u64 - size_of::<MCFGTable>() as u64) / size_of::<PCIDeviceConfiguration>() as u64;
        MCFGIterator {
            address,
            end_address: address + entries,
        }
    }
}

impl From<&'static ACPITable> for &MCFGTable {
    fn from(table: &'static ACPITable) -> Self {
        unsafe { (table as *const ACPITable as *const MCFGTable).as_ref().unwrap() }
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

pub fn bar_to_struct<T>(bar: u32) -> &'static T {
    unsafe {
        ((bar as u64 + mem::PHYSICAL_MEMORY_OFFSET) as *const T).as_ref().unwrap()
    }
}

pub fn bar_to_struct_64<T>(bar_l: u32, bar_h: u32) -> &'static mut T {
    unsafe {
        (((((bar_h as u64) << 32) | bar_l as u64) as u64 + mem::PHYSICAL_MEMORY_OFFSET) as *mut T).as_mut().unwrap()
    }
}