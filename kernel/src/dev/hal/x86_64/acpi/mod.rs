use self::tables::{RSDPHeader, ACPITable};
use crate::{*, dev::hal::pci::{MCFGTable}};

pub mod tables;

pub static mut RSDP_ADDRESS: u64 = 0;

pub fn init() {
    let rsdp = RSDPHeader::from_address(unsafe { RSDP_ADDRESS });
    let rxsdt = match rsdp.revision {
        0 => ACPITable::from_address(rsdp.rsdt_address as u64),
        _ => ACPITable::from_address(rsdp.xsdt_address),
    };
    let mcfg: &MCFGTable = rxsdt.get_table("MCFG").unwrap().into();
    for conf in mcfg {
        for bus in conf {
            for dev in bus {
                for func in dev {
                    let head = func.device_header();
                    let ven = head.vendor_id;
                    let devi = head.device_id;
                    println!("{:x} {:x}", ven, devi);
                }
            }
        }
    }
}