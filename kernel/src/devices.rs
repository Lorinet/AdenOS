use alloc::collections::BTreeMap;

use crate::*;
use alloc::{boxed::Box, string::String};

static mut DEVICES: BTreeMap<String, Box<dyn dev::Device>> = BTreeMap::new();

pub fn register_device(device: impl dev::Device + 'static) {
    unsafe {
        DEVICES.insert(String::from(device.device_name()), Box::new(device));
    }
}

pub fn get_device(name: String) -> Option<&'static mut Box<dyn dev::Device>> {
    unsafe {
        DEVICES.get_mut(&name)
    }
}

pub fn get_devices() -> impl Iterator<Item = &'static String> {
    unsafe {
        DEVICES.keys().into_iter()
    }
}