use alloc::{slice, vec, vec::Vec};
use modular_bitfield::{bitfield, specifiers::*};

use crate::dev::storage::AHCIDrive;
use crate::{*, dev::Read};
use super::*;
use alloc::string::String;
use alloc::boxed::Box;

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
    fn read_table<T>(device: &'static mut T) -> Result<MBRPartitionTable, Error>
    where T: ReadFrom {
        let mut tab = MBRPartitionTable {
            table: [MBRPartition::new(); 4],
        };
        device.read_from(unsafe { &mut *(&mut tab.table as *mut _ as *mut [u8; 64]) }, 0x1BE)?;
        Ok(tab)
    }
}