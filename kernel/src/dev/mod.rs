use crate::*;
use console::ConsoleColor;
use alloc::fmt::Debug;

pub mod char;
pub mod hal;
pub mod power;
pub mod input;
pub mod storage;
pub mod framebuffer;
pub mod filesystem;

#[derive(Debug)]
pub enum Error {
    InitFailure(&'static str),
    DeinitFailure(&'static str),
    IOFailure(&'static str),
    ReadFailure,
    WriteFailure,
    DriverNotFound(&'static str),
}

pub trait Device {
    fn init_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn deinit_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn device_name(&self) -> &str;
}

pub trait StaticDevice {
    fn init_device() -> Result<(), Error> { Ok(()) }
    fn deinit_device() -> Result<(), Error> { Ok(()) }
    fn device_name() -> &'static str;
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
    fn seek_relative(&mut self, offset: usize);
    fn seek_begin(&mut self);
    fn seek_end(&mut self);
    fn offset(&mut self) -> usize;
}

pub trait SeekRead: Device + Seek + Read + ReadFrom {}
pub trait SeekReadWrite: Device + Seek + Read + ReadFrom + Write + WriteTo {}

pub trait ReadFrom: Seek + Read {
    fn read_from(&mut self, buf: &mut [u8], offset: usize) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.read(buf);
        self.seek(prev_offset);
        result
    }
}

pub trait WriteTo: Seek + Write {
    fn write_to(&mut self, buf: &[u8], offset: usize) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.write(buf);
        self.seek(prev_offset);
        result
    }
}

impl<T> ReadFrom for T where T: Seek + Read {}
impl<T> WriteTo for T where T: Seek + Write {}

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