use crate::*;
use console::ConsoleColor;

pub mod char;
pub mod hal;
pub mod power;
pub mod input;
pub mod storage;
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
    type T: Copy;
    fn write_one(&mut self, val: Self::T) -> Result<(), Error>;
    fn write(&mut self, buf: &[Self::T]) -> Result<usize, Error> {
        for v in buf {
            self.write_one(*v)?;
        }
        Ok(buf.len())
    }
}

pub trait Read {
    type T: Copy;
    fn read_one(&mut self) -> Result<Self::T, Error>;
    fn read(&mut self, buf: &mut [Self::T]) -> Result<usize, Error> {
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

pub trait ReadFrom: Seek + Read {
    fn read_from(&mut self, buf: &mut [Self::T], offset: usize) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.read(buf);
        self.seek(prev_offset);
        result
    }
}

pub trait WriteTo: Seek + Write {
    fn write_to(&mut self, buf: &[Self::T], offset: usize) -> Result<usize, Error> {
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