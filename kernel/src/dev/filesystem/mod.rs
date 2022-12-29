use alloc::boxed::Box;

use super::*;

pub mod fat;

pub trait FileSystem: Resource {
    fn volume_label(&self) -> String;
    fn create_file(&self, path: String) -> Result<Box<dyn RandomReadWrite + '_>, Error>;
    fn open_file(&self, path: String) -> Result<Box<dyn RandomReadWrite + '_>, Error>;
}
