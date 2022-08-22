use self::tables::{RSDPHeader, ACPITable};

use crate::{*, dev::hal::pci::MCFGTable};

pub mod tables;

pub static mut RSDP_ADDRESS: u64 = 0;

pub fn init() {
    let rsdp = RSDPHeader::from_address(unsafe { RSDP_ADDRESS });
    serial_println!("{:#?}", rsdp);
    let rxsdt = match rsdp.revision {
        0 => ACPITable::from_address(rsdp.rsdt_address as u64),
        _ => ACPITable::from_address(rsdp.xsdt_address),
    };
    let mcfg: &MCFGTable = rxsdt.get_table("MCFG").unwrap().into();
    serial_println!("MCFG: {:#?}", mcfg);
    for ent in mcfg {
        serial_println!("{:#?}", ent);
    }
}