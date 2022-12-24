use crate::namespace::ResourceType;

use super::*;
use alloc::boxed::Box;
use core::str;

#[derive(Debug)]
enum FATType {
    FAT12,
    FAT16,
    FAT32,
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct BPB {
    _jmp: [u8; 3],
    oem_id: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sector_count: u16,
    file_allocation_tables: u8,
    root_directory_entries: u16,
    sector_count: u16,
    media_descriptor_type: u8,
    fat_size: u16,
    _sectors_per_track: u16,
    _heads_sides: u16,
    hidden_sector_count: u32,
    large_sector_count: u32,
    extension: [u8; 54],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct eBPB16 {
    drive_number: u8,
    _reserved: u8,
    extended_boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fat_type: [u8; 5],
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
struct eBPB32 {
    fat_size: u32,
    flags: u16,
    fat_version: u16,
    root_directory_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    _reserved: [u8; 12],
    extension: eBPB16,
}

pub struct FATFileSystem {
    drive: &'static mut dyn BlockReadWrite,
    fat_type: FATType,
    total_sectors: u64,
    fat_size: u64,
    root_dir_sectors: u64,
    first_data_sector: u64,
    first_fat_sector: u64,
    data_sectors: u64,
    total_clusters: u64,
    root_dir_sector: u64,
}

impl Debug for FATFileSystem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FATFileSystem")
        .field("drive", &self.drive.resource_path_string())
        .field("fat_type", &self.fat_type)
        .field("total_sectors", &self.total_sectors)
        .field("fat_size", &self.fat_size)
        .field("root_dir_sectors", &self.root_dir_sectors)
        .field("first_data_sector", &self.first_data_sector)
        .field("first_fat_sector", &self.first_fat_sector)
        .field("data_sectors", &self.data_sectors)
        .field("total_clusters", &self.total_clusters)
        .field("root_dir_sector", &self.root_dir_sector)
        .finish()
    }
}

impl FATFileSystem {
    pub fn new(drive_path: String) -> Result<Option<Self>, Error> {
        if let Some(drive) = namespace::get_block_device(drive_path.clone()) {
            let mut bpb = [0; 512];
            drive.read_block(0, bpb.as_mut_ptr())?;
            let bpb = unsafe { (&bpb as *const _ as *const BPB).as_ref().unwrap() };
            let ebpb16 = unsafe { (&bpb.extension as *const _ as *const eBPB16).as_ref().unwrap() };
            let ebpb32 = unsafe { (&bpb.extension as *const _ as *const eBPB32).as_ref().unwrap() };

            let total_sectors = match bpb.sector_count {
                0 => bpb.large_sector_count as u64,
                _ => bpb.sector_count as u64,
            };
            let fat_size = match bpb.fat_size {
                0 => ebpb32.fat_size as u64,
                _ => bpb.fat_size as u64,
            };
            let root_dir_sectors = (((bpb.root_directory_entries * 32) + (bpb.bytes_per_sector - 1)) / bpb.bytes_per_sector) as u64;
            let first_data_sector = bpb.reserved_sector_count as u64 + (bpb.file_allocation_tables as u64 * fat_size) + root_dir_sectors;
            let first_fat_sector = bpb.reserved_sector_count as u64;
            let data_sectors = bpb.sector_count as u64 - (bpb.reserved_sector_count as u64 + (bpb.file_allocation_tables as u64 * fat_size) + root_dir_sectors);
            let total_clusters = data_sectors / bpb.sectors_per_cluster as u64;

            let fat_type = if total_clusters < 4085 {
                FATType::FAT12
            } else if total_clusters < 65525 {
                FATType::FAT16
            } else {
                FATType::FAT32
            };

            let (signature, check_signature) = match fat_type {
                FATType::FAT12 => (&ebpb16.fat_type, "FAT12"),
                FATType::FAT16 => (&ebpb16.fat_type, "FAT16"),
                FATType::FAT32 => (&ebpb32.extension.fat_type, "FAT32"),
            };

            // check signature
            if let Ok(signature) = str::from_utf8(signature) {
                if signature != check_signature {
                    return Ok(None);
                }
            } else {
                return Ok(None);
            }

            let root_dir_sector = match fat_type {
                FATType::FAT32 => ebpb32.root_directory_cluster as u64 * bpb.sectors_per_cluster as u64,
                _ => first_data_sector - root_dir_sectors,
            };

            Ok(Some(FATFileSystem {
                drive,
                fat_type,
                total_sectors,
                fat_size,
                root_dir_sectors,
                first_data_sector,
                first_fat_sector,
                data_sectors,
                total_clusters,
                root_dir_sector
            }))
        } else {
            Err(Error::InvalidDevice(drive_path))
        }
    }
}

impl FileSystem for FATFileSystem {
    fn volume_label(&self) -> String {
        self.drive.resource_path().last().unwrap().clone()    
    }

    fn open_file(&mut self, path: String) -> Result<Box<dyn RandomReadWrite>, Error> {
        Err(Error::IOFailure(""))
    }
}

impl Resource for FATFileSystem {
    fn unwrap(&mut self) -> namespace::ResourceType {
        namespace::ResourceType::FileSystem(self)
    }

    fn resource_path(&self) -> Vec<String> {
        vec![String::from("Files"), self.volume_label()]
    }
}