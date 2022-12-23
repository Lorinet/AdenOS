use crate::*;
use console::ConsoleColor;
use namespace::Resource;
use alloc::{fmt::Debug, string::String, vec, vec::Vec};

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
    ReadFailure,
    WriteFailure,
    DriverNotFound(&'static str),
}

pub trait Device: Resource + Debug {
    fn init_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn deinit_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn device_path(&self) -> Vec<String>;
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

pub trait Seek {
    fn seek(&mut self, position: usize);
    fn seek_relative(&mut self, offset: isize);
    fn seek_begin(&mut self);
    fn seek_end(&mut self);
    fn offset(&mut self) -> usize;
}

pub trait RandomRead: Seek + Read {
    fn read_from(&mut self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.read(buf);
        self.seek(prev_offset);
        result
    }
}

pub trait RandomWrite: Seek + Write {
    fn write_to(&mut self, buf: &[u8], offset: usize) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.write(buf);
        self.seek(prev_offset);
        result
    }
}

impl<T> RandomRead for T where T: Device + Seek + Read {}
impl<T> RandomWrite for T where T: Device + Seek + Write {}

pub trait RandomReadWrite: RandomRead + RandomWrite {}
impl<T> RandomReadWrite for T where T: RandomRead + RandomWrite {}

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