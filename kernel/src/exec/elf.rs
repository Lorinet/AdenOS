use crate::*;
use super::*;
use dev::*;
use infinity::io::*;
use alloc::{vec, vec::Vec};

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
enum ELFInstructionSet {
    NonSpecific = 0,
    Sparc = 2,
    x86 = 3,
    MIPS = 8,
    PowerPC = 0x14,
    ARM = 0x28,
    SuperOne = 0x2A,
    x86_64 = 0x3E,
    AArch64 = 0xB7,
    RISCV = 0xF3,
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
enum ELFExecutableType {
    Relocatable = 1,
    Executable = 2,
    Shared = 3,
    Core = 4,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ELFBitness {
    ELF32 = 1,
    ELF64 = 2,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ELFEndianness {
    Little = 1,
    Big = 2,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ELFHeader {
    signature: u32,
    bitness: ELFBitness,
    endianness: ELFEndianness,
    header_version: u8,
    abi: u8,
    _reserved: u64,
    executable_type: ELFExecutableType,
    instruction_set: ELFInstructionSet,
    elf_version: u32,
    extension: [u8; 40],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ELF32HeaderExt {
    entry_point: u32,
    program_header_table_offset: u32,
    section_header_table_offset: u32,
    flags: u32,
    header_size: u16,
    program_header_entry_size: u16,
    program_header_entry_count: u16,
    section_header_entry_size: u16,
    section_header_entry_count: u16,
    section_name_table_index: u16,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ELF64HeaderExt {
    entry_point: u64,
    program_header_table_offset: u64,
    section_header_table_offset: u64,
    flags: u32,
    header_size: u16,
    program_header_entry_size: u16,
    program_header_entry_count: u16,
    section_header_entry_size: u16,
    section_header_entry_count: u16,
    section_name_table_index: u16,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
enum ELFSectionType {
    Null = 0,
    Load = 1,
    Dynamic = 2,
    Interpreter = 3,
    Note = 4,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
enum ELFSectionFlags {
    Executable = 1,
    Writable = 2,
    Readable = 4,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ProgramHeaderEntry32 {
    section_type: ELFSectionType,
    file_offset: u32,
    virt_address: u32,
    _reserved: u32,
    size_in_file: u32,
    size_in_memory: u32,
    flags: ELFSectionFlags,
    required_alignment: u32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct ProgramHeaderEntry64 {
    section_type: ELFSectionType,
    flags: ELFSectionFlags,
    file_offset: u64,
    virt_address: u64,
    _reserved: u64,
    size_in_file: u64,
    size_in_memory: u64,
    required_alignment: u64,
}

pub struct ELFLoader {}

impl ELFLoader {
    pub fn load_executable(handle: u32) -> Result<ExecutableInfo, Error> {
        if let Some(file) = namespace::get_file_handle(handle) {
            let mut buf = vec![0; 64];
            file.seek(0)?;
            file.read(&mut buf)?;
            let head0 = unsafe { (buf.as_ptr() as *const ELFHeader).as_ref().unwrap() };
            serial_println!("{:#x?}", head0);
            
            if head0.signature != 0x464C457F { // ELF signature
                return Err(Error::InvalidExecutable);
            }

            let mut entry_point = 0;
            let mut sections = Vec::new();

            if let ELFBitness::ELF64 = head0.bitness {
                let head1 = unsafe { (head0.extension.as_ptr() as *const ELF64HeaderExt).as_ref().unwrap() };
                serial_println!("{:#?}", head1);
                entry_point = head1.entry_point as usize;
                for i in 0..head1.program_header_entry_count {
                    let mut buf = vec![0; head1.program_header_entry_size as usize];
                    file.seek(head1.program_header_table_offset + (i * head1.program_header_entry_size) as u64)?;
                    file.read(&mut buf)?;
                    let phdr = unsafe { (buf.as_ptr() as *const ProgramHeaderEntry64).as_ref().unwrap() };
                    if let ELFSectionType::Load = phdr.section_type {
                        sections.push(Section {
                            section_type: SectionType::Load,
                            file_offset: phdr.file_offset,
                            size_in_file: phdr.size_in_file as usize,
                            virt_address: phdr.virt_address as usize,
                            size_in_memory: phdr.size_in_memory as usize,
                        });
                    } // other sections later
                }
            } else {
                return Err(Error::InvalidExecutable);
            }

            Ok(ExecutableInfo {
                file_handle: handle,
                virt_entry_point: entry_point,
                sections,
            })
        } else {
            Err(Error::InvalidHandle)
        }
    }
}