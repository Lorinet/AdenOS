use crate::{*, dev::{*, storage::Drive}};
use alloc::{vec, string::{String, ToString}, vec::Vec};

use super::{AHCI, DiskIO};

pub struct AHCIDrive {
    driver: Option<&'static mut AHCI>,
    port: usize,
    name: String,
    offset: usize,
    end: usize,
}

impl AHCIDrive {
    pub fn new(port: usize) -> AHCIDrive {
        AHCIDrive {
            driver: None,
            port,
            name: String::from("Drives/AHCI") + &port.to_string(),
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

    fn seek_relative(&mut self, offset: usize) {
        self.offset += offset;
    }
}

impl Read for AHCIDrive {
    fn read_one(&mut self) -> Result<u8, dev::Error> {
        let sector = self.offset / 512;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        self.driver.as_mut().unwrap().ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, 1, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = self.offset % 512;
        Ok(buffer[buf_off])
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, dev::Error> {
        let sector = self.offset / 512;
        let count = (((self.offset + buf.len()) / 512) - sector) + 1;
        let mut buffer: Vec<u8> = vec![0; count * 512];
        self.driver.as_mut().unwrap().ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = self.offset % 512;
        buf.copy_from_slice(&buffer[buf_off..buf_off + buf.len()]);
        Ok(buf.len())
    }
}

impl Write for AHCIDrive {
    fn write_one(&mut self, val: u8) -> Result<(), dev::Error> {
        let sector = self.offset / 512;
        let mut buffer: Vec<u8> = Vec::with_capacity(512);
        self.driver.as_mut().unwrap().ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, 1, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = self.offset % 512;
        buffer[buf_off] = val;
        self.driver.as_mut().unwrap().ports[self.port].expect("Disk not present").diskio(DiskIO::Write, sector as u64, 1, buffer.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, dev::Error> {
        let sector = self.offset / 512;
        let count = (((self.offset + buf.len()) / 512) - sector) + 1;
        let mut buffer: Vec<u8> = vec![0; count * 512];
        self.driver.as_mut().unwrap().ports[self.port].expect("Disk not present").diskio(DiskIO::Read, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::ReadFailure)?;
        let buf_off = self.offset % 512;
        buffer[buf_off..buf_off + buf.len()].copy_from_slice(&buf[0..buf.len()]);
        self.driver.as_mut().unwrap().ports[self.port].expect("Disk not present").diskio(DiskIO::Write, sector as u64, count as u16, buffer.as_mut_ptr()).map_err(|_| dev::Error::WriteFailure)?;
        Ok(buf.len())
    }
}

impl Drive for AHCIDrive {
    fn capacity(&mut self) -> usize {
        0
    }
}

impl Device for AHCIDrive {
    fn init_device(&mut self) -> Result<(), Error> {
        if let Some(driver) = devices::get_device::<AHCI>(String::from("Storage/AHCI")) {
            self.driver = Some(driver);
            Ok(())
        } else {
            Err(Error::DriverNotFound("Storage/AHCI"))
        }
    }

    fn deinit_device(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn device_name(&self) -> &str {
        &self.name
    }
}