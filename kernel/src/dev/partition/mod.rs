use crate::namespace::ResourceType;

use super::*;
use alloc::{string::{String, ToString}, vec::Vec};

pub mod mbr;
pub mod gpt;

pub trait PartitionTable {
    fn read_partitions(drive_path: String) -> Result<Option<Vec<Partition>>, Error>;
}

pub enum PartitionType {
    EFISystemPartition,
    DataPartition,
}

pub struct Partition {
    drive_path: Vec<String>,
    drive: Option<&'static mut dyn BlockReadWrite>,
    partition_name: String,
    partition_label: String,
    start_sector: u64,
    end_sector: u64,
    sector_offset: u64,
    partition_type: PartitionType,
}

impl Debug for Partition {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Partition")
        .field("drive_path", &namespace::concat_resource_path(self.drive_path.clone()))
        .field("partition_name", &self.partition_name)
        .field("start_sector", &self.start_sector)
        .field("end_sector", &self.end_sector)
        .field("sector_offset", &self.sector_offset)
        .finish()
    }
}

impl Partition {
    fn mount_file_system(&mut self) -> Result<(), Error> {
        let fat = filesystem::fat::FATFileSystem::new(self.resource_path_string())?;
        if let Some(fat) = fat {
            serial_println!("{:#?}", namespace::register_resource(fat));
            return Ok(())
        }
        Ok(())
    }
}

impl Device for Partition {
    fn init_device(&mut self) -> Result<(), Error> {
        if let Some(drive) = namespace::get_block_device_parts(self.drive_path.clone()) {
            let _ = self.drive.insert(drive);
            self.mount_file_system()?;
            Ok(())
        } else {
            return Err(Error::InvalidDevice)
        }
    }

    fn device_path(&self) -> Vec<String> {
        let mut drivepath = self.drive_path.clone();
        drivepath.remove(0);
        drivepath.push(self.partition_name.clone());
        drivepath
    }

    fn unwrap(&mut self) -> DeviceClass {
        DeviceClass::BlockDevice(self)
    }
}

impl Seek for Partition {
    fn offset(&self) -> u64 {
        self.sector_offset
    }

    fn seek(&mut self, position: u64) -> Result<(), Error> {
        if self.end_sector <= position {
            return Err(Error::InvalidSeek);
        }
        self.sector_offset = position;
        Ok(())
    }

    fn size(&self) -> u64 {
        self.end_sector - self.start_sector
    }
}

impl Read for Partition {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.drive.as_mut().unwrap().seek(self.start_sector * 512 + self.sector_offset * 512);
        self.drive.as_mut().unwrap().read(buf)
    }
}

impl Write for Partition {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.drive.as_mut().unwrap().seek(self.start_sector + self.sector_offset);
        self.drive.as_mut().unwrap().write(buf)
    }
}

impl BlockRead for Partition {
    fn block_size(&self) -> usize {
        self.drive.as_ref().unwrap().block_size()
    }

    fn read_block(&mut self, block: u64, buffer: *mut u8) -> Result<(), Error> {
        self.drive.as_mut().unwrap().read_block(self.start_sector + block, buffer)
    }

    fn read_blocks(&mut self, start_block: u64, count: u64, buffer: *mut u8) -> Result<(), Error> {
        self.drive.as_mut().unwrap().read_blocks(self.start_sector + start_block, count, buffer)
    }
}

impl BlockWrite for Partition {
    fn write_block(&mut self, block: u64, buffer: &mut [u8]) -> Result<(), Error> {
        self.drive.as_mut().unwrap().write_block(self.start_sector + block, buffer)
    }

    fn write_blocks(&mut self, start_block: u64, buffer: &mut [u8]) -> Result<(), Error> {
        self.drive.as_mut().unwrap().write_blocks(self.start_sector + start_block, buffer)
    }
}