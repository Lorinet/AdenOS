use alloc::{string::String, vec::Vec};

use crate::dev::Error;

use self::elf::ELFLoader;

pub mod elf;
pub mod scheduler;

enum SectionType {
    Load,
    Dynamic,
    Interpreter,
}

pub struct Section {
    section_type: SectionType,
    file_offset: u64,
    size_in_file: usize,
    virt_address: usize,
    size_in_memory: usize,
}

pub struct ExecutableInfo {
    file_handle: u32,
    virt_entry_point: usize,
    sections: Vec<Section>,
}

pub struct ExecutableLoader {}

impl ExecutableLoader {
    pub fn load_executable(handle: u32) -> Result<ExecutableInfo, Error> {
        ELFLoader::load_executable(handle)
    }
}