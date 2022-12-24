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
    fn read_one(&mut self) -> Result<u8, dev::Error> {
        Ok(self.port.receive())
    }
}

impl dev::Write for Uart16550 {
    fn write_one(&mut self, val: u8) -> Result<(), dev::Error> {
        self.port.write_char(val as char).or_else(|_| Result::Err(dev::Error::WriteFailure))
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
