use crate::*;
use alloc::{string::String, vec::Vec};
use self::elf::ELFLoader;

pub mod elf;
pub mod scheduler;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum SectionType {
    Load,
    Dynamic,
    Interpreter,
}

#[derive(Copy, Clone, Debug)]
pub struct Section {
    pub section_type: SectionType,
    pub file_offset: u64,
    pub size_in_file: usize,
    pub virt_address: usize,
    pub size_in_memory: usize,
}

#[derive(Clone, Debug)]
pub struct ExecutableInfo {
    pub file_handle: u32,
    pub virt_entry_point: usize,
    pub sections: Vec<Section>,
}

pub struct ExecutableLoader {}

impl ExecutableLoader {
    pub fn load_executable(handle: u32) -> Result<ExecutableInfo, Error> {
        ELFLoader::load_executable(handle)
    }
}