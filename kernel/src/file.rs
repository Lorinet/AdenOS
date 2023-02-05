use crate::{*, dev::{filesystem::FileSystem}, namespace::{ResourceType, Resource, Handle}};
use infinity::io::*;
use alloc::{boxed::Box, string::{String, ToString}};
use bitflags::bitflags;
use alloc::{vec, vec::Vec};

bitflags! {
    pub struct FilePermissions: u16 {
        const READ = 0b0000000000000001;
        const WRITE = 0b0000000000000010;
        const EXECUTE = 0b0000000000000100;
    }
}

pub struct File {
    fs: &'static dyn FileSystem,
    fs_file: Box<dyn RandomReadWrite>,
    path: String,
    permissions: FilePermissions,
    is_open: bool,
}

impl File {
    pub fn new(path: String, fs: &'static dyn FileSystem, fs_file: Box<dyn RandomReadWrite>, permissions: FilePermissions) -> File {
        File {
            path,
            fs,
            fs_file,
            permissions,
            is_open: false,
        }
    }

    pub fn open(path: String) -> Result<&'static mut Handle, Error> {
        let parts = namespace::split_resource_path(path.clone());
        if let Some(r) = parts.get(0) {
            if *r == "Files" {
                if let Some(root) = parts.get(1) {
                    if let Some(fs) = namespace::get_resource_non_generic(String::from("Files/") + root) {
                        if let ResourceType::FileSystem(fs) = fs.unwrap() {
                            let mut newpath = String::new();
                            for part in &parts[2..] {
                                newpath += (String::from("/") + part.as_str()).as_str();
                            }
                            serial_println!("Newpath: {}", newpath);
                            let file = fs.open_file(newpath)?;
                            serial_println!("Opened file");
                            if let Some(_) = namespace::get_resource_non_generic(file.resource_path_string()) {
                                Err(Error::AlreadyOpen)
                            } else {
                                serial_println!("Path should be {}", file.resource_path_string());
                                namespace::register_resource(file);
                                serial_println!("Registered resource");
                                namespace::acquire_handle(path, 0)
                            }
                        } else {
                            Err(Error::EntryNotFound)
                        }
                    } else {
                        Err(Error::EntryNotFound)
                    }
                } else {
                    Err(Error::EntryNotFound)
                }
            } else {
                Err(Error::EntryNotFound)
            }
        } else {
            Err(Error::EntryNotFound)
        }
    }
}

impl Resource for File {
    fn is_open(&self) -> bool {
        self.is_open
    }

    fn set_open_state(&mut self, open: bool) {
        self.is_open = open;
        if open == false {
            namespace::drop_resource_parts(self.resource_path()).unwrap();
        }
    }

    fn resource_path(&self) -> Vec<String> {
        namespace::split_resource_path(self.path.clone())
    }

    fn unwrap(&mut self) -> ResourceType {
        ResourceType::File(self)
    }
}

impl Seek for File {
    fn offset(&self) -> u64 {
        self.fs_file.offset()
    }

    fn seek(&mut self, position: u64) -> Result<(), Error> {
        self.fs_file.seek(position)
    }

    fn seek_begin(&mut self) -> Result<(), Error> {
        self.fs_file.seek_begin()
    }

    fn seek_end(&mut self) -> Result<(), Error> {
        self.fs_file.seek_end()
    }

    fn seek_relative(&mut self, offset: i64) -> Result<(), Error> {
        self.fs_file.seek_relative(offset)
    }

    fn size(&self) -> u64 {
        self.fs_file.size()
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.fs_file.read(buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.fs_file.write(buf)
    }
}