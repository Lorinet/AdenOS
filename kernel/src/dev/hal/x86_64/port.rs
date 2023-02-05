use crate::{*, namespace::Resource};
use dev;
use x86_64;
use x86_64::instructions::port;
use alloc::{vec, vec::Vec, string::{String, ToString}};

#[derive(Debug)]
pub struct Port {
    number: u16,
    port: port::Port<u8>,
}

impl Port {
    pub const fn new(number: u16) -> Port {
        Port {
            number,
            port: port::Port::new(number),
        }
    }
}

impl dev::Device for Port {
    fn device_path(&self) -> Vec<String> {
        //format!("HAL/Port{:#06x}", self.number).as_str()
        vec![String::from("System"), String::from("Ports"), self.number.to_string()]
    }

    fn unwrap(&mut self) -> dev::DeviceClass {
        dev::DeviceClass::ReadWriteDevice(self)
    }
}

impl dev::Read for Port {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        for b in buf.iter_mut() {
            *b = unsafe { self.port.read() } as u8;
        }
        Ok(buf.len())
    }
}

impl dev::Write for Port {
    fn write(&mut self, buf: &[u8]) -> Result<usize, dev::Error> {
        for val in buf {
            unsafe {
                self.port.write(*val);
            }
        }
        Ok(buf.len())
    }
}