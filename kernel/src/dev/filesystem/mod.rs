use alloc::boxed::Box;

use super::*;

pub mod fat;

pub trait FileSystem: Resource {
    fn volume_label(&self) -> String;
    fn open_file(&mut self, path: String) -> Result<Box<dyn RandomReadWrite>, Error>;
}
