use self::structures::{RSDPHeader, ACPITable};

use crate::*;

pub mod structures;

pub static mut RSDP_ADDRESS: u64 = 0;

pub fn init() {
    let rsdp = RSDPHeader::from_address(unsafe { RSDP_ADDRESS });
    serial_println!("{:#?}", rsdp);
    let rxsdt = match rsdp.revision {
        0 => ACPITable::from_address(rsdp.rsdt_address as u64),
        _ => ACPITable::from_address(rsdp.xsdt_address),
    };
    serial_println!("{:#?}", rxsdt);
    for sdt in rxsdt {
        serial_println!("{:#?}", sdt);
    }
}