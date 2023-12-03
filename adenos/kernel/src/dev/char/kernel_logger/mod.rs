use alloc::{vec, vec::Vec, string::String};
use infinity::Error;

use crate::{dev::{DeviceClass, Device, Write, Read}, println, namespace::{Resource, ResourceType}};

#[derive(Debug)]
pub struct KernelLogger;

impl KernelLogger {
    pub const fn new() -> KernelLogger {
        KernelLogger
    }
}

impl Device for KernelLogger {
    fn init_device(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("Character"), String::from("KernelLogger")]
    }

    fn is_in_use(&self) -> bool {
        false
    }

    fn unwrap(&mut self) -> DeviceClass {
        DeviceClass::ReadWriteDevice(self)
    }
}

impl Write for KernelLogger {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        println!("{}", alloc::str::from_utf8(buf).unwrap());
        Ok(buf.len())
    }
}

impl Read for KernelLogger {
    fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Error> {
        Err(Error::NoData)
    }
}