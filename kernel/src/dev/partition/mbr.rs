use alloc::{vec::Vec, string::String};
use modular_bitfield::{bitfield, specifiers::*};
use super::*;

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct MBRPartition {
    boot: u8,
    starting_head: u8,
    starting_sector: B6,
    starting_cylinder: B10,
    system_id: u8,
    ending_head: u8,
    ending_sector: B6,
    ending_cylinder: B10,
    relative_sector: u32,
    total_sectors: u32,
}

pub struct MBRPartitionTable {
    pub table: [MBRPartition; 4],
}

impl PartitionTable for MBRPartitionTable {
    fn read_partitions(device_path: String) -> Result<Option<Vec<Partition>>, Error> {
        let mut table = [MBRPartition::new(); 4];
        if let Some(device) = namespace::get_block_device(device_path.clone()) {
            device.read_from(unsafe { &mut *(&mut table as *mut _ as *mut [u8; 64]) }, 0x1BE)?;
            let mut parts = Vec::<Partition>::new();
            for i in 0..4 {
                let mbrp = &table[i];
                if mbrp.system_id() == 0 && mbrp.total_sectors() == 0 {
                    continue;
                }
                let ss = u32::from_le(mbrp.relative_sector()) as u64;
                let es = ss + u32::from_le(mbrp.total_sectors()) as u64 - 1;
                parts.push(Partition {
                    drive_path: device.resource_path(),
                    drive: None,
                    partition_name: String::from("Partition") + i.to_string().as_str(),
                    partition_label: String::from("Partition") + i.to_string().as_str(),
                    start_sector: ss,
                    end_sector: es,
                    sector_offset: 0,
                    partition_type: PartitionType::DataPartition,
                });
            }
            Ok(Some(parts))
        } else {
            return Err(Error::DriverNotFound(device_path))
        }
    }
}