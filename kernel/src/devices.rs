use crate::{*, collections::tree::Tree, dev::*};
use alloc::{boxed::Box, string::String, vec::Vec};

static mut DEVICE_TREE: Tree<String, Box<dyn dev::Device>> = Tree::new(String::new(), None);

pub fn get_device_tree() -> &'static mut Tree<String, Box<dyn dev::Device>> {
    unsafe {
        &mut DEVICE_TREE
    }
}

pub fn cast_device<D>(device: &'static mut Box<dyn Device>) -> &'static mut D {
    unsafe {
        ((*device).as_mut() as *mut _ as *mut D).as_mut().unwrap()
    }
}

pub fn device_tree() -> &'static mut Tree<String, Box<dyn dev::Device>> {
    unsafe {
        &mut DEVICE_TREE
    }
}

pub fn get_device<D>(path: Vec<String>) -> &'static mut D {
    cast_device::<D>(device_tree().get_node_by_path(path).unwrap().value().unwrap())
}

pub fn register_device(device: impl Device + 'static) {
    device_tree().insert_node_by_path(device.device_path(), Some(Box::new(device)));
}

pub fn register_device_path(path: Vec<String>, device: impl Device + 'static) {
    device_tree().insert_node_by_path(path, Some(Box::new(device)));
}