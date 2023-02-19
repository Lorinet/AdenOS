use crate::*;
use dev::*;
use file::*;

pub mod fat;

pub trait FileSystem: Resource {
    fn volume_label(&self) -> String;
    fn create_file(&self, path: String) -> Result<File, Error>;
    fn open_file(&self, path: String) -> Result<File, Error>;
}