use crate::*;
use dev::hal::port;
use dev::Write;

enum QemuExitCode {
    Success = 0x10,
    Failure = 0x11,
}

pub struct QemuExit {
    port: port::Port<u32>,
}

impl QemuExit {
    pub fn new() -> QemuExit {
        QemuExit {
            port: port::Port::new(0xf4)
        }
    }
}

impl dev::Device for QemuExit {
    fn device_name(&self) -> &str {
        "Power/QemuExit"
    }
}

impl dev::PowerControl for QemuExit {
    fn reboot(&mut self) -> ! {
        self.port.write_one(&(QemuExitCode::Failure as u32)).unwrap();
        loop {}
    }
    fn shutdown(&mut self) -> ! {
        self.port.write_one(&(QemuExitCode::Success as u32)).unwrap();
        loop {}
    }
}