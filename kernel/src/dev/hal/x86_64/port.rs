use crate::*;
use dev;
use x86_64;
use x86_64::instructions::port;

pub struct Port {
    port: port::Port<u8>,
}

impl Port {
    pub const fn new(number: u16) -> Port {
        Port {
            port: port::Port::new(number),
        }
    }
}

impl dev::Device for Port {
    fn device_name(&self) -> &str {
        //format!("HAL/Port{:#06x}", self.number).as_str()
        "HAL/Port"
    }
}

impl dev::Read for Port {
    fn read_one(&mut self) -> Result<u8, dev::Error> {
        unsafe {
            Ok(self.port.read() as u8)
        }
    }
}

impl dev::Write for Port {
    fn write_one(&mut self, val: u8) -> Result<(), dev::Error> {
        unsafe {
            self.port.write(val);
        }
        Ok(())
    }
    fn write(&mut self, buf: &[u8]) -> Result<usize, dev::Error> {
        for val in buf {
            self.write_one(*val)?;
        }
        Ok(buf.len())
    }
}