use alloc::{vec::Vec, string::String, format};
use core::{str, mem::size_of, slice, ops::ControlFlow};
use bitflags::bitflags;
use modular_bitfield::{bitfield, specifiers::*};
use super::{*, mbr::MBRPartitionTable};

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct GUID {
    time_low: u32,
    time_mid: u16,
    time_hi_and_version: u16,
    clock_seq_hi_and_res_clock_seq_low: u16,
    node: B48,
}

impl GUID {
    pub fn is_zero(&self) -> bool {
        self.time_low() == 0 && self.time_mid() == 0 && self.time_hi_and_version() == 0 && self.clock_seq_hi_and_res_clock_seq_low() == 0 && self.node() == 0
    }

    pub fn to_string(&self) -> String {
        let mut str = String::new();
        str += format!("{:08X}", self.time_low()).as_str();
        str += "-";
        str += format!("{:04X}", self.time_mid()).as_str();
        str += "-";
        str += format!("{:04X}", self.time_hi_and_version()).as_str();
        str += "-";
        str += format!("{:04X}", self.clock_seq_hi_and_res_clock_seq_low().to_be()).as_str();
        str += "-";
        str += format!("{:012X}", self.node().to_be() / 0x10000).as_str();
        str
    }
}

bitflags! {
    pub struct GPTAttributes: u64 {
        const FIRMWARE = 0b0000000000000001;
        const USED_BY_OS = 0b0000000000000010;
        const LEGACY_BOOT = 0b0000000000000100;
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct GPTHeader {
    signature: [u8; 8],
    gpt_revision: u32,
    header_size: u32,
    header_checksum: u32,
    _reserved_0: u32,
    primary_header_lba: u64,
    alternate_header_lba: u64,
    first_usable_block: u64,
    last_usable_block: u64,
    disk_guid: GUID,
    partition_table_start_lba: u64,
    partition_entry_count: u32,
    partition_entry_size: u32,
    partition_table_checksum: u32,
    _reserved_1: [u8; 0x200 - 0x5C],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct GPTPartition {
    partition_type_guid: GUID,
    partition_guid: GUID,
    start_lba: u64,
    end_lba: u64,
    attributes: GPTAttributes,
}

pub struct GPTPartitionTable {}

impl PartitionTable for GPTPartitionTable {
    fn read_partitions(drive_path: String) -> Result<Option<Vec<Partition>>, Error> {
        if let Some(drive) = namespace::get_block_device(drive_path.clone()) {
            let mut gpt_header = [0; 512];
            drive.read_block(1, gpt_header.as_mut_ptr())?;
            let gpt_header = unsafe { (&gpt_header as *const _ as *const GPTHeader).as_ref().unwrap() };
            if let Ok(signature) = str::from_utf8(&gpt_header.signature) {
                if signature == "EFI PART" {
                    let table_size = (gpt_header.partition_entry_size * gpt_header.partition_entry_count) as usize;
                    let mut entry_buffer = vec![0; table_size];
                    drive.read_blocks(2, table_size as u64 / 512, entry_buffer.as_mut_ptr())?;
                    let mut partitions = Vec::<Partition>::new();
                    let name_length = gpt_header.partition_entry_size as usize - size_of::<GPTPartition>();
                    for i in 0..gpt_header.partition_entry_count {
                        let gpt_part = unsafe { (entry_buffer.as_ptr().offset((i * gpt_header.partition_entry_size) as isize) as *const GPTPartition).as_ref().unwrap() };
                        if !gpt_part.partition_type_guid.is_zero() {
                            let mut name_slice = unsafe { slice::from_raw_parts((gpt_part as *const _ as *const u16).offset((size_of::<GPTPartition>() as isize) / 2), name_length / 2) };
                            let mut name_length_actual = name_length / 2;
                            for i in 0..name_length / 2 {
                                if name_slice[i] == 0 {
                                    name_length_actual = i;
                                    break;
                                }
                            }
                            name_slice = &name_slice[..name_length_actual];
                            if let Ok(name) = String::from_utf16(name_slice) { 
                                partitions.push(Partition {
                                    drive_path: namespace::split_resource_path(drive_path.clone()),
                                    drive: None,
                                    partition_name: gpt_part.partition_guid.to_string(),
                                    partition_label: String::from(name),
                                    start_sector: gpt_part.start_lba,
                                    end_sector: gpt_part.end_lba,
                                    sector_offset: 0,
                                    partition_type: match gpt_part.partition_type_guid.to_string().as_str() {
                                        "C12A7328-F81F-11D2-BA4B-00A0C93EC93B" => PartitionType::EFISystemPartition,
                                        _ => PartitionType::DataPartition,
                                    },
                                });
                            }
                        }
                    }
                    Ok(Some(partitions))
                } else {
                    Ok(None)
                }
            } else {
                return Ok(None);
            }
        } else {
            return Err(Error::InvalidDevice)
        }
    }
}