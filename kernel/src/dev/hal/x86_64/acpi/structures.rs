use crate::*;
use core::{mem::size_of, str, fmt};
use crate::dev::hal::mem;

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct ACPITableSignature {
    bytes: [u8; 4],
}

impl fmt::Debug for ACPITableSignature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", str::from_utf8(&self.bytes).expect("INVALID_ACPI_TABLE"))
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct RSDPHeader {
    pub signature: [u8; 8],
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub revision: u8,
    pub rsdt_address: u32,
    pub length: u32,
    pub xsdt_address: u64,
    pub extended_checksum: u8,
    _reserved: [u8; 3],
}

impl RSDPHeader {
    pub fn from_address(phys_address: u64) -> &'static RSDPHeader {
        unsafe { ((phys_address + mem::PHYSICAL_MEMORY_OFFSET) as *const RSDPHeader).as_ref().unwrap() }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy, Debug)]
pub struct ACPITable {
    pub signature: ACPITableSignature,
    pub length: u32,
    pub revision: u8,
    pub checksum: u8,
    pub oem_id: [u8; 6],
    pub oem_table_id: [u8; 8],
    pub oem_revision: u32,
    pub creator_id: u32,
    pub creator_revision: u32,
}

impl ACPITable {
    pub fn from_address(phys_address: u64) -> &'static ACPITable {
        unsafe { ((phys_address + mem::PHYSICAL_MEMORY_OFFSET) as *const ACPITable).as_ref().unwrap() }
    }

    pub fn entry_count(&self) -> usize {
        ((self.length as usize) - size_of::<ACPITable>()) / 8
    }

    pub fn iter(&self) -> ACPITableIterator {
        self.into_iter()
    }
}

impl IntoIterator for &ACPITable {
    type Item = &'static ACPITable;
    type IntoIter = ACPITableIterator;

    fn into_iter(self) -> Self::IntoIter {
        ACPITableIterator {
            address: ((self as *const _ as usize) + size_of::<ACPITable>()) as u64,
            end_address: ((self as *const _ as usize) + (self.entry_count() * size_of::<ACPITable>())) as u64,
            entry_size: match str::from_utf8(&self.signature.bytes).expect("INVALID_ACPI_TABLE") {
                "XSDT" => 8,
                _ => 4,
            }
        }
    }
}

pub struct ACPITableIterator {
    address: u64,
    end_address: u64,
    entry_size: u8,
}

impl Iterator for ACPITableIterator {
    type Item = &'static ACPITable;
    fn next(&mut self) -> Option<Self::Item> {
        if self.address > self.end_address - size_of::<ACPITable>() as u64 {
            None
        } else {
            let address = unsafe {
                let address = match self.entry_size {
                    8 => *(self.address as *const u64),
                    _ => *(self.address as *const u32) as u64,
                };
                address + mem::PHYSICAL_MEMORY_OFFSET
            };
            serial_println!("ACPI Table Address: {:x}", address);
            let sdt = unsafe { (address as *const ACPITable).as_ref().unwrap() };
            self.address += self.entry_size as u64;
            Some(sdt)
        }
    }
}

#[repr(C)]
#[repr(packed)]
pub struct MCFGHeader {
    pub sdt_header: ACPITable,
    _reserved: u64,
}