use crate::*;
use crate::namespace::Resource;
use alloc::{vec, vec::Vec, string::String};
use dev::hal::port;
use dev::Write;

enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

#[derive(Debug)]
pub struct QemuExit {
    port: port::Port,
}

impl QemuExit {
    pub fn new() -> QemuExit {
        QemuExit {
            port: port::Port::new(0xf4)
        }
    }
}

impl dev::Device for QemuExit {
    fn device_path(&self) -> Vec<String> {
        vec![String::from("Power"), String::from("QemuExit")]
    }
}

impl dev::PowerControl for QemuExit {
    fn reboot(&mut self) -> ! {
        self.port.write_one(QemuExitCode::Failure as u8).unwrap();
        loop {}
    }
    fn shutdown(&mut self) -> ! {
        self.port.write_one(QemuExitCode::Success as u8).unwrap();
        loop {}
    }
}