use crate::*;
use self::tables::{RSDPHeader, ACPITable};
use dev::{hal::{apic::{madt::*, lapic::*, ioapic::*}, pci::*}, storage};
use alloc::{format, string::*};

pub mod tables;

pub static mut RSDP_ADDRESS: u64 = 0;

#[allow(unused_variables)]
pub fn init() {
    let rsdp = RSDPHeader::from_address(unsafe { RSDP_ADDRESS });
    let rxsdt = match rsdp.revision {
        0 => ACPITable::from_address(rsdp.rsdt_address as u64),
        _ => ACPITable::from_address(rsdp.xsdt_address),
    };

    let madt: &MADTTable = rxsdt.get_table("APIC").unwrap().into();
    devices::register_device(LAPIC::new(madt.lapic_address, madt.flags & 1 > 0));
    let mut ioapics = 0;
    for apic in madt {
        match apic.entry_type {
            MADTEntryType::LAPIC => {},
            MADTEntryType::IOAPIC => {
                let ioapic_ent: &MADTEntryIOAPIC = apic.into();
                devices::register_device(IOAPIC::new(ioapic_ent.ioapic_address, ioapic_ent.ioapic_id, ioapic_ent.global_system_interrupt_base));
                ioapics += 1;
            },
            MADTEntryType::IOAPICInterruptSourceOverride => {
                let ioapic_ent: &MADTEntryIOAPICInterruptSourceOverride = apic.into();
                let mut the_ioapic_we_need = devices::get_device::<IOAPIC>(String::from("System/IOAPIC0")).expect(&format!("IOAPIC 0 not found."));
                for i in 0..ioapics {
                    let ioapic = devices::get_device::<IOAPIC>(String::from("System/IOAPIC") + &i.to_string()).expect(&format!("IOAPIC {} not found.", i));
                    if ioapic.interrupt_base <= ioapic_ent.global_system_interrupt {
                        the_ioapic_we_need = ioapic;
                    } else {
                        break;
                    }
                }
                the_ioapic_we_need.interrupt_source_overrides.push(ioapic_ent.clone());
            },
            MADTEntryType::IOAPICNMISource => {},
            MADTEntryType::LAPICNonMaskableInterrupts => {},
            MADTEntryType::LAPICAddressOverride => {
                let over_ent: &MADTEntryLAPICAddressOverride = apic.into();
                let addr = over_ent.lapic_base_address;
                panic!("LAPIC address is actually at {:#x}", addr);
            },
            MADTEntryType::X2LAPIC => {},
        }
    }

    let mcfg: &MCFGTable = rxsdt.get_table("MCFG").unwrap().into();
    for conf in mcfg {
        for bus in conf {
            for dev in bus {
                for func in dev {
                    let head = func.device_header();
                    let class = head.class;
                    let subclass = head.subclass;
                    let prog_if = head.prog_if;
                    let vendor_id = head.vendor_id;
                    let device_id = head.device_id;
                    //println!("{} / {} / {} / {:x}", pci::id::get_vendor_name(vendor_id), pci::id::get_class_name(class), pci::id::get_subclass_name(class, subclass), device_id);
                    match class {
                        0x01 => match subclass {     // Mass storage controller
                            0x06 => match prog_if {  // SATA controller
                                0x01 => {            // AHCI 1.0
                                    devices::register_device(storage::AHCI::new(head));
                                }
                                _ => (),
                            },
                            0x08 => match prog_if {  // NVM controller
                                0x02 => {            // NVMe
                                    devices::register_device(storage::NVME::new(head));
                                }
                                _ => (),
                            },
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
        }
    }
}