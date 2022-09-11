use crate::*;
use alloc::{boxed::Box, string::String, collections::BTreeMap, vec::Vec};

static mut DEVICES: BTreeMap<String, Box<dyn dev::Device>> = BTreeMap::new();

pub fn register_device(device: impl dev::Device + 'static) -> String {
    let name = String::from(device.device_name());
    unsafe {
        DEVICES.insert(name.clone(), Box::new(device));
    }
    name
}

pub fn get_device_non_generic(name: String) -> Option<&'static mut Box<dyn dev::Device>> {
    unsafe {
        DEVICES.get_mut(&name)
    }
}

pub fn get_device<D>(name: String) -> Option<&'static mut D> {
    unsafe {
        DEVICES.get_mut(&name).map(|dev| ((&**dev) as *const _ as *mut D).as_mut().unwrap())
    }
}

pub fn get_devices() -> impl Iterator<Item = String> {
    unsafe {
        DEVICES.keys().cloned().collect::<Vec<String>>().into_iter()
    }
}