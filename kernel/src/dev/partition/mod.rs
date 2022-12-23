use super::*;
use super::storage::Drive;
use alloc::{string::{String, ToString}, vec::Vec};

pub mod mbr;

pub trait PartitionTable {
    fn read_partitions<T>(device: &'static mut T) -> Result<Vec<Partition>, Error>
    where T: Drive, Self: Sized;
}

pub struct Partition {
    drive: &'static mut dyn Drive,
    partition_number: usize,
    start_sector: usize,
    end_sector: usize,
    sector_offset: usize,
}

impl Debug for Partition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Partition")
        .field("drive", &self.drive.resource_path_string())
        .field("partition_number", &self.partition_number)
        .field("start_address", &self.start_sector)
        .field("end_address", &self.end_sector)
        .field("offset", &self.sector_offset)
        .finish()
    }
}

impl Device for Partition {
    fn device_path(&self) -> Vec<String> {
        let mut drivepath = self.drive.device_path();
        drivepath.push(String::from("Partition") + &self.partition_number.to_string());
        drivepath
    }
}

impl Seek for Partition {
    fn offset(&mut self) -> usize {
        self.sector_offset
    }

    fn seek(&mut self, position: usize) {
        self.sector_offset = position;
    }

    fn seek_begin(&mut self) {
        self.sector_offset = 0;
    }

    fn seek_end(&mut self) {
        self.sector_offset = self.end_sector - self.start_sector - 1;
    }

    fn seek_relative(&mut self, offset: isize) {
        self.sector_offset = ((self.sector_offset as isize) + offset) as usize;
    }
}

impl Read for Partition {
    fn read_one(&mut self) -> Result<u8, Error> {
        self.drive.seek(self.start_sector + self.sector_offset);
        self.drive.read_one()
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.drive.seek(self.start_sector + self.sector_offset);
        self.drive.read(buf)
    }
}

impl Write for Partition {
    fn write_one(&mut self, val: u8) -> Result<(), Error> {
        self.drive.seek(self.start_sector + self.sector_offset);
        self.drive.write_one(val)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.drive.seek(self.start_sector + self.sector_offset);
        self.drive.write(buf)
    }
}
