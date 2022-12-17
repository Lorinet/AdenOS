use super::*;
use super::storage::Drive;
use alloc::string::String;

pub mod mbr;

pub trait PartitionTable {
    fn read_table<T>(device: &'static mut T) -> Result<Self, Error>
    where T: ReadFrom, Self: Sized;
}

pub struct Partition {
    drive: &'static dyn Drive,
    start_address: usize,
    end_address: usize,
}