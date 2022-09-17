use crate::*;
use dev;
use uart_16550;
use core::fmt::{self, Write};

pub struct Uart16550 {
    pub number: u8,
    port: uart_16550::SerialPort,
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
    fn device_name(&self) -> &str {
        "Character/Uart16550"
    }
}

impl dev::Read for Uart16550 {
    type T = u8;
    fn read_one(&mut self) -> Result<Self::T, dev::Error> {
        Ok(self.port.receive())
    }
}

impl dev::Write for Uart16550 {
    type T = u8;
    fn write_one(&mut self, val: Self::T) -> Result<(), dev::Error> {
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
