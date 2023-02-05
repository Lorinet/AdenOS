use crate::{*, dev::{*, partition::PartitionTable}};
use alloc::{vec, string::{String, ToString}, vec::Vec};

use super::{AHCI, DiskIO};

#[derive(Debug)]
pub struct AHCIDrive {
    controller: &'static mut AHCI,
    port: usize,
    offset: u64,
    end: u64,
}

impl AHCIDrive {
    pub fn new(controller: &'static mut AHCI, port: usize) -> AHCIDrive {
        AHCIDrive {
            controller,
            port,
            offset: 0,
            end: 0,
        }
    }
}

impl Seek for AHCIDrive {
    fn offset(&self) -> u64 {
        self.offset
    }

    fn seek(&mut self, position: u64) -> Result<(), Error> {
        if self.end < position {
            return Err(Error::InvalidSeek);
        }
        self.offset = position;
        Ok(())
    }

    fn size(&self) -> u64 {
        self.end + 1
    }
}

impl Read for AHCIDrive {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, dev::Error> {
        let sector = self.offset / 512;
        let count = (((self.offset + buf.len() as u64) / 512) - sector) + 1;
        let mut buffer = vec![0; (count * 512) as usize];
        self.read_blocks(sector, count, buffer.as_mut_ptr())?;
        let buf_off = (self.offset % 512) as usize;
        buf.copy_from_slice(&buffer[buf_off..buf_off + buf.len()]);
        Ok(buf.len())
    }
}

impl Write for AHCIDrive {
    fn write(&mut self, buf: &[u8]) -> Result<usize, dev::Error> {
        let sector = self.offset / 512;
        let count = (((self.offset + buf.len() as u64) / 512) - sector) + 1;
        let mut buffer: Vec<u8> = vec![0; (count * 512) as usize];
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = (self.offset % 512) as usize;
        buffer[buf_off..buf_off + buf.len()].copy_from_slice(&buf[0..buf.len()]);
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Write, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(buf.len())
    }
}

impl BlockRead for AHCIDrive {
    fn block_size(&self) -> usize {
        512
    }

    fn read_block(&mut self, block: u64, buffer: *mut u8) -> Result<(), Error> {
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Read, block as u64, 1, buffer).map_err(|_| dev::Error::ReadFailure)?;
        Ok(())
    }

    fn read_blocks(&mut self, start_block: u64, count: u64, buffer: *mut u8) -> Result<(), Error> {
        for i in start_block..(start_block + count) {
            self.read_block(i, unsafe { buffer.offset(((i - start_block) * 512) as isize) })?;
        }
        Ok(())
    }
}

impl BlockWrite for AHCIDrive {
    fn write_block(&mut self, block: u64, buf: &mut [u8]) -> Result<(), Error> {
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Write, block as u64, 1, buf.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(())
    }

    fn write_blocks(&mut self, start_block: u64, buf: &mut [u8]) -> Result<(), Error> {
        for i in start_block..(start_block + (buf.len() as u64 / self.block_size() as u64)) {
            self.write_block(i, &mut buf[((i - start_block) * self.block_size() as u64) as usize..((i + 1) * self.block_size() as u64) as usize])?;
        }
        Ok(())
    }
}

impl Device for AHCIDrive {
    fn init_device(&mut self) -> Result<(), Error> {
        let partitions = match partition::gpt::GPTPartitionTable::read_partitions(self.resource_path_string())? {
            Some(partitions) => partitions,
            None => match partition::mbr::MBRPartitionTable::read_partitions(self.resource_path_string())? {
                Some(partitions) => partitions,
                None => Vec::new(),
            },
        };
        for partition in partitions {
            namespace::register_resource(partition).init_device()?;
        }
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("Storage"), String::from("AHCI"), String::from("Drive") + self.port.to_string().as_str()]
    }

    fn unwrap(&mut self) -> DeviceClass {
        DeviceClass::BlockDevice(self)
    }
}