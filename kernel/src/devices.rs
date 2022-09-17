use crate::*;
use alloc::{boxed::Box, string::String, collections::BTreeMap, vec::Vec, format};

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

pub fn init_device(device: String) -> Result<(), dev::Error> {

    let device_name = device.clone();
    get_device_non_generic(device).expect(format!("Device not found: {}", device_name.as_str()).as_str()).init_device()
}

pub fn deinit_device(device: String) -> Result<(), dev::Error> {

    let device_name = device.clone();
    get_device_non_generic(device).expect(format!("Device not found: {}", device_name.as_str()).as_str()).deinit_device()
}