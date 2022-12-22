use crate::{*, dev::{*, storage::{Drive}}};
use alloc::{vec, string::{String, ToString}, vec::Vec};

use super::{AHCI, DiskIO};

#[derive(Debug)]
pub struct AHCIDrive {
    controller: &'static mut AHCI,
    port: usize,
    offset: usize,
    end: usize,
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
    fn offset(&mut self) -> usize {
        self.offset
    }

    fn seek(&mut self, position: usize) {
        self.offset = position;
    }

    fn seek_begin(&mut self) {
        self.offset = 0;
    }

    fn seek_end(&mut self) {
        self.offset = self.end;
    }

    fn seek_relative(&mut self, offset: isize) {
        self.offset = ((self.offset as isize) + offset) as usize;
    }
}

impl Read for AHCIDrive {
    fn read_one(&mut self) -> Result<u8, dev::Error> {
        Ok(self.read_sector(self.offset / 512)?[self.offset % 512])
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, dev::Error> {
        let sector = self.offset / 512;
        let count = (((self.offset + buf.len()) / 512) - sector) + 1;
        let buffer = self.read_sectors(sector, count)?;
        println!("Sector bytes: {}", buffer.len());
        let buf_off = self.offset % 512;
        buf.copy_from_slice(&buffer[buf_off..buf_off + buf.len()]);
        Ok(buf.len())
    }
}

impl Write for AHCIDrive {
    fn write_one(&mut self, val: u8) -> Result<(), dev::Error> {
        let sector = self.offset / 512;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, 1, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = self.offset % 512;
        buffer[buf_off] = val;
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Write, sector as u64, 1, buffer.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, dev::Error> {
        let sector = self.offset / 512;
        let count = (((self.offset + buf.len()) / 512) - sector) + 1;
        let mut buffer: Vec<u8> = vec![0; count * 512];
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = self.offset % 512;
        buffer[buf_off..buf_off + buf.len()].copy_from_slice(&buf[0..buf.len()]);
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Write, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(buf.len())
    }
}

impl Drive for AHCIDrive {
    fn capacity(&mut self) -> usize {
        0
    }

    fn sector_size(&self) -> usize {
        512
    }

    fn read_sector(&mut self, sector: usize) -> Result<Vec<u8>, Error> {
        let mut buf = vec![0; self.sector_size()];
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, 1, buf.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        Ok(buf)
    }

    fn read_sectors(&mut self, start_sector: usize, count: usize) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::<u8>::with_capacity(count * self.sector_size());
        for i in start_sector..(start_sector + count) {
            buf.extend_from_slice(&self.read_sector(i)?);
        }
        Ok(buf)
    }

    fn write_sector(&mut self, sector: usize, buf: &mut [u8]) -> Result<(), Error> {
        self.controller.ports[self.port].expect("Disk not present").diskio(DiskIO::Write, sector as u64, 1, buf.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(())
    }

    fn write_sectors(&mut self, start_sector: usize, buf: &mut [u8]) -> Result<(), Error> {
        for i in start_sector..(start_sector + (buf.len() / self.sector_size())) {
            self.write_sector(i, &mut buf[i * self.sector_size()..(i + 1) * self.sector_size()])?;
        }
        Ok(())
    }
}

impl Device for AHCIDrive {
    fn init_device(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("Storage"), String::from("AHCI"), String::from("Drive") + self.port.to_string().as_str()]
    }
}