use crate::*;
use console::ConsoleColor;
use namespace::Resource;
use alloc::{fmt::Debug, string::String, vec, vec::Vec};

use self::{framebuffer::Framebuffer, filesystem::FileSystem};

pub mod char;
pub mod hal;
pub mod power;
pub mod input;
pub mod storage;
pub mod partition;
pub mod filesystem;
pub mod framebuffer;

#[derive(Debug)]
pub enum Error {
    InitFailure(&'static str),
    DeinitFailure(&'static str),
    IOFailure(&'static str),
    OutOfSpace,
    ReadFailure,
    WriteFailure,
    InvalidData,
    InvalidDevice(String),
    DriverNotFound(String),
}

pub enum DeviceClass<'a> {
    ReadDevice(&'a mut dyn Read),
    WriteDevice(&'a mut dyn Write),
    ReadWriteDevice(&'a mut dyn ReadWrite),
    RandomReadWriteDevice(&'a mut dyn RandomReadWrite),
    Framebuffer(&'a mut dyn Framebuffer),
    BlockDevice(&'a mut dyn BlockReadWrite),
    Other,
}

pub trait Device: Resource + Debug {
    fn init_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn deinit_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn device_path(&self) -> Vec<String>;
    fn unwrap(&mut self) -> DeviceClass;/* {
        DeviceClass::Other
    }*/
}

impl<T: Device> Resource for T {
    fn resource_path(&self) -> Vec<String> {
        [vec![String::from("Devices")], self.device_path()].concat()
    }

    fn unwrap(&mut self) -> namespace::ResourceType {
        namespace::ResourceType::Device(self)
    }
}

pub trait StaticDevice {
    fn init_device() -> Result<(), Error> { Ok(()) }
    fn deinit_device() -> Result<(), Error> { Ok(()) }
    fn device_path() -> Vec<String>;
}

pub trait Write {
    fn write_one(&mut self, val: u8) -> Result<(), Error>;
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        for v in buf {
            self.write_one(*v)?;
        }
        Ok(buf.len())
    }
}

pub trait Read {
    fn read_one(&mut self) -> Result<u8, Error>;
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        for i in 0..buf.len() {
            buf[i] = self.read_one()?;
        }
        Ok(buf.len())
    }
}

pub trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

pub trait Seek {
    fn seek(&mut self, position: u64);
    fn offset(&self) -> u64;
    fn seek_begin(&mut self);
    fn seek_end(&mut self);
    fn seek_relative(&mut self, offset: i64) {
        self.seek(((self.offset() as i64) + offset) as u64);
    }
}

pub trait RandomRead: Seek + Read {
    fn read_from(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.read(buf);
        self.seek(prev_offset);
        result
    }
}

pub trait RandomWrite: Seek + Write {
    fn write_to(&mut self, buf: &[u8], offset: u64) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.write(buf);
        self.seek(prev_offset);
        result
    }
}

impl<T> RandomRead for T where T: Seek + Read {}
impl<T> RandomWrite for T where T: Seek + Write {}

pub trait RandomReadWrite: RandomRead + RandomWrite {}
impl<T> RandomReadWrite for T where T: RandomRead + RandomWrite {}

pub trait BlockRead: Device {
    fn block_size(&self) -> usize;
    fn read_block(&mut self, block: u64, buffer: *mut u8) -> Result<(), Error>;
    fn read_blocks(&mut self, start_block: u64, count: u64, buffer: *mut u8) -> Result<(), Error>;
}

pub trait BlockWrite: BlockRead {
    fn write_block(&mut self, block: u64, buffer: &mut [u8]) -> Result<(), Error>;
    fn write_blocks(&mut self, start_block: u64, buffer: &mut [u8]) -> Result<(), Error>;
}

pub trait BlockReadWrite: RandomRead + RandomWrite + BlockRead + BlockWrite {}
impl<T: RandomRead + RandomWrite + BlockRead + BlockWrite> BlockReadWrite for T {}

pub trait PowerControl {
    fn shutdown(&mut self) -> ! {
        loop {}
    }
    fn reboot(&mut self) -> ! {
        loop {}
    }
}

pub trait ConsoleDevice {
    fn buffer_size(&self) -> (i32, i32);
    fn clear_screen(&mut self);
    fn set_color(&mut self, foreground: ConsoleColor, background: ConsoleColor);
}