use crate::*;
use dev;
use x86_64;
use x86_64::instructions::port;

pub struct Port<T> {
    number: u16,
    port: port::Port<T>,
}

impl<T> Port<T> {
    pub const fn new(number: u16) -> Port<T> {
        Port {
            number,
            port: port::Port::new(number),
        }
    }
}

impl<T> dev::Device for Port<T> {
    fn device_name(&self) -> &str {
        //format!("HAL/Port{:#06x}", self.number).as_str()
        "HAL/Port"
    }
}

impl<T> dev::Read for Port<T>
where T: port::PortRead + Copy {
    type T = T;
    fn read_one(&mut self) -> Result<Self::T, dev::Error> {
        unsafe {
            Ok(self.port.read() as Self::T)
        }
    }
}

impl<T> dev::Write for Port<T>
where T: port::PortWrite + Copy {
    type T = T;
    fn write_one(&mut self, val: Self::T) -> Result<(), dev::Error> {
        unsafe {
            self.port.write(val);
        }
        Ok(())
    }
    fn write(&mut self, buf: &[Self::T]) -> Result<usize, dev::Error> {
        for val in buf {
            self.write_one(*val)?;
        }
        Ok(buf.len())
    }
}