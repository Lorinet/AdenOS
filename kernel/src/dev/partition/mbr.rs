use alloc::{vec::Vec};
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
    fn read_partitions<T>(device: &'static mut T) -> Result<Vec<Partition>, Error>
    where T: Drive {
        let mut table = [MBRPartition::new(); 4];
        device.read_from(unsafe { &mut *(&mut table as *mut _ as *mut [u8; 64]) }, 0x1BE)?;
        let mut parts = Vec::<Partition>::new();
        for i in 0..4 {
            let mbrp = &table[i];
            if mbrp.system_id() == 0 && mbrp.total_sectors() == 0 {
                continue;
            }
            let ss = u32::from_le(mbrp.relative_sector()) as usize;
            let es = ss + u32::from_le(mbrp.total_sectors()) as usize - 1;
            parts.push(Partition {
                drive: unsafe { (device as *mut dyn Drive).as_mut().unwrap() },
                partition_number: i,
                start_sector: ss,
                end_sector: es,
                sector_offset: 0,
            });
        }
        Ok(parts)
    }
}