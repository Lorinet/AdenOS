use crate::{*, namespace::Resource};
use alloc::{vec, vec::Vec, string::String};
use dev;
use uart_16550;
use core::fmt::{self, Write, Debug};

pub struct Uart16550 {
    pub number: u8,
    port: uart_16550::SerialPort,
}

impl Debug for Uart16550 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Uart16550").field("number", &self.number).finish()
    }
}

impl Uart16550 {
    pub const fn new(number: u8) -> Uart16550 {
        Uart16550 {
            number,
            port: unsafe { uart_16550::SerialPort::new(0x3F8 + number as u16) },
        }
    }
}

impl dev::Device for Uart16550 {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        self.port.init();
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("Character"), String::from("Uart16550")]
    }

    fn unwrap(&mut self) -> dev::DeviceClass {
        dev::DeviceClass::ReadWriteDevice(self)
    }
}

impl dev::Read for Uart16550 {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        for b in buf.iter_mut() {
            *b = self.port.receive()
        }
        Ok(buf.len())
    }
}

impl dev::Write for Uart16550 {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        for b in buf {
            self.port.write_char(*b as char).or_else(|_| return Result::Err(dev::Error::WriteFailure));
        }
        Ok(buf.len())
    }
}

impl fmt::Write for Uart16550 {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        dev::Write::write(self, s.as_bytes()).unwrap();
        Ok(())
    }
}

impl dev::ConsoleDevice for Uart16550 {
    fn buffer_size(&self) -> (i32, i32) {
        (0, 0)
    }

    fn clear_screen(&mut self) {
        
    }

    fn set_color(&mut self, _foreground: console::ConsoleColor, _background: console::ConsoleColor) {
        
    }
}

unsafe impl Send for Uart16550 {}
