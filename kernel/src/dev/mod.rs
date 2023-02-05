use crate::*;
use console::ConsoleColor;
use namespace::Resource;
use alloc::{fmt::Debug, string::String, vec, vec::Vec};
use infinity::io::*;
use self::{framebuffer::Framebuffer, filesystem::FileSystem};

pub mod char;
pub mod hal;
pub mod power;
pub mod input;
pub mod storage;
pub mod partition;
pub mod filesystem;
pub mod framebuffer;

impl<T: Device> Resource for T {
    fn resource_path(&self) -> Vec<String> {
        [vec![String::from("Devices")], self.device_path()].concat()
    }

    fn unwrap(&mut self) -> namespace::ResourceType {
        namespace::ResourceType::Device(self)
    }
}

pub trait StaticDevice {
    fn init_device() -> Result<(), Error> { Ok(()) }
    fn deinit_device() -> Result<(), Error> { Ok(()) }
    fn device_path() -> Vec<String>;
}

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