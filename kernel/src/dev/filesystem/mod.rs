use alloc::boxed::Box;

use crate::file::File;

use super::*;

pub mod fat;

pub trait FileSystem: Resource {
    fn volume_label(&self) -> String;
    fn create_file(&self, path: String) -> Result<File, Error>;
    fn open_file(&self, path: String) -> Result<File, Error>;
}