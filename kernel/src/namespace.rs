use crate::{*, collections::tree::Tree, dev::{*, filesystem::FileSystem}, ipc::MessageQueue};
use alloc::{boxed::Box, string::String, vec, vec::Vec, collections::BTreeMap};
use infinity::io::*;

pub enum ResourceType<'a> {
    Device(&'a mut dyn Device),
    FileSystem(&'a mut dyn FileSystem),
    File(&'a mut file::File),
    MessageQueue(&'a mut MessageQueue),
    Other
}

pub trait Resource {
    fn is_open(&self) -> bool {
        true // do not allow handle acquisition if not implemented
    }

    fn set_open_state(&mut self, open: bool) {

    }

    fn unwrap(&mut self) -> ResourceType;
    fn resource_path(&self) -> Vec<String>;
    fn resource_path_string(&self) -> String {
        let mut str = String::new();
        for node in self.resource_path() {
            str += (String::from("/") + node.as_str()).as_str();
        }
        str
    }
}

pub struct Handle {
    pub id: u32,
    pub owner: u32,
    resource: &'static mut Box<dyn Resource>,
}

impl Handle {
    pub fn new(id: u32, owner: u32, resource: &'static mut Box<dyn Resource>) -> Handle {
        Handle {
            id,
            owner,
            resource,
        }
    }

    pub fn unwrap(&'static mut self) -> &'static mut Box<dyn Resource> {
        self.resource
    }

    pub fn release(self) -> Result<(), Error> {
        namespace::release_handle(self.id)
    }
}

static mut NAMESPACE: Tree<String, Box<dyn Resource>> = Tree::new(String::new(), None);
static mut HANDLES: BTreeMap<u32, Handle> = BTreeMap::new();
static mut HANDLE_ID: u32 = 0;

pub fn cast_resource<D>(resource: &'static mut Box<dyn Resource>) -> &'static mut D {
    unsafe {
        ((*resource).as_mut() as *mut _ as *mut D).as_mut().unwrap()
    }
}

pub fn namespace() -> &'static mut Tree<String, Box<dyn Resource>> {
    unsafe {
        &mut NAMESPACE
    }
}

pub fn subtree_parts(path: Vec<String>) -> Option<&'static mut Tree<String, Box<dyn Resource>>> {
    namespace().get_node_by_path(path)
}

pub fn subtree(path: String) -> Option<&'static mut Tree<String, Box<dyn Resource>>> {
    namespace().get_node_by_path(split_resource_path(path))
}

pub fn init_namespace() {
    namespace().insert_subtree(String::from("Devices"), None);
    namespace().insert_subtree(String::from("Files"), None);
}

pub fn split_resource_path(path: String) -> Vec<String> {
    path.split("/").filter(|s| !s.is_empty()).map(|s| String::from(s)).collect()
}

pub fn concat_resource_path(path: Vec<String>) -> String {
    path.iter().map(|s| String::from("/") + s.as_str()).collect()
}

pub fn get_resource_parts<D>(path: Vec<String>) -> Option<&'static mut D> {
    get_resource_non_generic_parts(path).map(|d| cast_resource(d))
}

pub fn get_resource<D>(path: String) -> Option<&'static mut D> {
    get_resource_parts(split_resource_path(path))
}

pub fn get_resource_non_generic(path: String) -> Option<&'static mut Box<dyn Resource>> {
    get_resource_non_generic_parts(split_resource_path(path))
}

pub fn get_resource_non_generic_parts(path: Vec<String>) -> Option<&'static mut Box<dyn Resource>> {
    if let Some(node) = namespace().get_node_by_path(path) {
        node.value()
    } else {
        None
    }
}

pub fn register_resource<T: Resource + 'static>(resource: T) -> &'static mut T {
    let path = resource.resource_path();
    namespace().insert_node_by_path(path.clone(), Some(Box::new(resource)));
    get_resource_parts(path).unwrap()
}

pub fn register_resource_path<T: Resource + 'static>(path: Vec<String>, resource: T) -> &'static mut T {
    namespace().insert_node_by_path(path.clone(), Some(Box::new(resource)));
    get_resource_parts(path).unwrap()
}

pub fn get_block_device(path: String) -> Option<&'static mut dyn BlockReadWrite> {
    get_block_device_parts(split_resource_path(path))
}

pub fn get_block_device_parts(path: Vec<String>) -> Option<&'static mut dyn BlockReadWrite> {
    if let Some(drive) = namespace::get_resource_non_generic_parts(path) {
        if let ResourceType::Device(drive) = drive.unwrap() {
            if let DeviceClass::BlockDevice(drive) = Device::unwrap(drive) {
                return Some(drive)
            }
        }
    }
    None
}

pub fn get_new_handle_id() -> u32 {
    unsafe {
        while HANDLES.contains_key(&HANDLE_ID) {
            HANDLE_ID += 1;
        }
        let ohid = HANDLE_ID;
        HANDLE_ID += 1;
        ohid
    }
}

pub fn acquire_handle(path: String, owner: u32) -> Result<&'static mut Handle, Error> {
    serial_println!("Code reached here (path: {})", path);
    if let Some(res) = get_resource_non_generic(path) {
        serial_println!("Found res");
        if res.is_open() {
            serial_println!("Perms err");
            Err(Error::Permissions)
        } else {
            serial_println!("Creating hancle");
            res.set_open_state(true);
            let id = get_new_handle_id();
            let hndl = Handle::new(id, owner, res);
            unsafe {
                HANDLES.insert(hndl.id, hndl);
                serial_println!("Created handle {}, next one is {}", id, HANDLE_ID);
                Ok(HANDLES.get_mut(&id).unwrap())
            }
        }
    } else {
        Err(Error::EntryNotFound)
    }
}

pub fn release_handle(id: u32) -> Result<(), Error> {
    if let Some(res) = unsafe { HANDLES.remove(&id) } {
        res.resource.set_open_state(false);
        unsafe {
            namespace::HANDLE_ID = res.id;
        }
        Ok(())
    } else {
        Err(Error::InvalidHandle)
    }
}

pub fn drop_resource(path: String) -> Result<(), Error> {
    drop_resource_parts(split_resource_path(path))
}

pub fn drop_resource_parts(path: Vec<String>) -> Result<(), Error> {
    unsafe {
        NAMESPACE.remove_node_by_path(path)
    }
}

pub fn get_rw_handle(handle: u32) -> Option<&'static mut dyn ReadWrite> {
    unsafe {
        if let Some(hndl) = HANDLES.get_mut(&handle) {
            match hndl.unwrap().unwrap() {
                ResourceType::File(file) => Some(file),
                ResourceType::MessageQueue(que) => Some(que),
                _ => None,
            }
        } else {
            None
        }
    }
}

pub fn get_seek_handle(handle: u32) -> Option<&'static mut dyn Seek> {
    unsafe {
        if let Some(hndl) = HANDLES.get_mut(&handle) {
            if let ResourceType::File(file) = hndl.unwrap().unwrap() {
                Some(file)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn get_file_handle(handle: u32) -> Option<&'static mut file::File> {
    unsafe {
        if let Some(hndl) = HANDLES.get_mut(&handle) {
            if let ResourceType::File(file) = hndl.unwrap().unwrap() {
                Some(file)
            } else {
                None
            }
        } else {
            None
        }
    }
}

pub fn get_message_queue_handle(handle: u32) -> Option<&'static mut ipc::MessageQueue> {
    unsafe {
        if let Some(hndl) = HANDLES.get_mut(&handle) {
            if let ResourceType::MessageQueue(que) = hndl.unwrap().unwrap() {
                Some(que)
            } else {
                None
            }
        } else {
            None
        }
    }
}