use crate::namespace::ResourceType;

use super::*;
use alloc::{boxed::Box, string::ToString};
use modular_bitfield::{bitfield, specifiers::*};
use core::{str, slice};
use bitflags::bitflags;
use core::cell::RefCell;
use core::num;

mod fat;
mod dir;

use fat::*;
use dir::*;

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

enum eBPB {
    eBPB16(eBPB16),
    eBPB32(eBPB32),
}

pub struct FATFileSystem {
    drive: RefCell<&'static mut dyn BlockReadWrite>,
    bpb: BPB,
    ebpb: eBPB,
    fat_type: FATType,
    total_sectors: u32,
    fat_size: u32,
    root_dir_sectors: u32,
    first_data_sector: u32,
    first_fat_sector: u32,
    data_sectors: u32,
    total_clusters: u32,
    root_dir_sector: u32,
    sector_size: u32,
    cluster_size: u32,
    sectors_per_cluster: u32,
}

impl Debug for FATFileSystem {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FATFileSystem")
        .field("drive", &self.drive.borrow().resource_path_string())
        .field("fat_type", &self.fat_type)
        .field("total_sectors", &self.total_sectors)
        .field("fat_size", &self.fat_size)
        .field("root_dir_sectors", &self.root_dir_sectors)
        .field("first_data_sector", &self.first_data_sector)
        .field("first_fat_sector", &self.first_fat_sector)
        .field("data_sectors", &self.data_sectors)
        .field("total_clusters", &self.total_clusters)
        .field("root_dir_sector", &self.root_dir_sector)
        .field("bytes_per_sector", &self.sector_size)
        .field("sectors_per_cluster", &self.sectors_per_cluster)
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

            if bpb._jmp[0] != 0xEB || bpb._jmp[2] != 0x90 {
                return Ok(None);
            }

            let total_sectors = match bpb.sector_count {
                0 => bpb.large_sector_count,
                _ => bpb.sector_count as u32,
            };
            let fat_size = match bpb.fat_size {
                0 => ebpb32.fat_size as u32,
                _ => bpb.fat_size as u32,
            };
            let root_dir_sectors = ((bpb.root_directory_entries as u32 * 32) + (bpb.bytes_per_sector as u32 - 1)) / bpb.bytes_per_sector as u32;
            let first_data_sector = bpb.reserved_sector_count as u32 + (bpb.file_allocation_tables as u32 * fat_size) + root_dir_sectors;
            let first_fat_sector = bpb.reserved_sector_count as u32;
            let data_sectors = match bpb.sector_count {
                0 => bpb.large_sector_count,
                c => c as u32,
            } - (bpb.reserved_sector_count as u32 + (bpb.file_allocation_tables as u32 * fat_size) + root_dir_sectors);
            let total_clusters = data_sectors / bpb.sectors_per_cluster as u32;
            let sector_size = drive.block_size() as u32;
            let cluster_size = bpb.sectors_per_cluster as u32 * sector_size;

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
                FATType::FAT32 => ebpb32.root_directory_cluster as u32 * bpb.sectors_per_cluster as u32,
                _ => first_data_sector - root_dir_sectors,
            };

            let ebpb = match fat_type {
                FATType::FAT32 => eBPB::eBPB32(ebpb32.clone()),
                _ => eBPB::eBPB16(ebpb16.clone()),
            };

            let mut fat_fs = FATFileSystem {
                drive: RefCell::new(drive),
                bpb: bpb.clone(),
                ebpb,
                fat_type,
                total_sectors,
                fat_size,
                root_dir_sectors,
                first_data_sector,
                first_fat_sector,
                data_sectors,
                total_clusters,
                root_dir_sector,
                sector_size,
                cluster_size,
                sectors_per_cluster: bpb.sectors_per_cluster as u32,
            };
            fat_fs.test_fs()?;

            Ok(Some(fat_fs))
        } else {
            Err(Error::InvalidDevice(drive_path))
        }
    }

    fn test_fs(&mut self) -> Result<(), Error> {
        //let mut file = self.create_file(String::from("Let's make an extremely long file.weed"), 36)?;
        //file.seek(0);
        //file.write("we are getting stoned as fuck here".as_bytes())?;
        let mut file = self.create_file(String::from("Holy shit how freaking dick.scss"), 100)?;
        file.seek(0);
        file.write("hjskldhksalhdjksadjkashdjk".as_bytes())?;
        
        //self.allocate_clusters(5);
        //return Ok(());
        for ent in self.root_dir_iter()? {
            serial_println!("{}  {} bytes", ent.name, ent.size);
            /*serial_println!("Start file {}", &ent.file_name);
            let mut file = File::new(self, ent.cluster, ent.size as usize)?;
            let mut buf = vec![0; 512];
            file.seek(0);
            file.read(buf.as_mut_slice())?;
            for b in buf {
                serial_print!("{}", b as char);
            }*/
        }
        Ok(())
    }

    fn cluster_to_sector(&self, cluster: u32) -> u32 {
        ((cluster - 2) * self.sectors_per_cluster) + self.first_data_sector
    }

    fn read_cluster(&self, cluster: u32, buffer: &mut [u8]) -> Result<(), Error> {
        self.drive.borrow_mut().read_blocks(self.cluster_to_sector(cluster) as u64, self.sectors_per_cluster as u64, buffer.as_mut_ptr())
    }

    fn fat_iter(&self, start_cluster: u32) -> Result<FATIterator, Error> {
        FATIterator::new(self, start_cluster)
    }

    fn root_dir_raw_iter(&self) -> Result<DirectoryRawIterator<'_>, Error> {
        match self.ebpb {
            eBPB::eBPB32(ebpb) => DirectoryRawIterator::new(self, Some(ebpb.root_directory_cluster)),
            _ => DirectoryRawIterator::new(self, None),
        }
    }

    fn root_dir_iter(&self) -> Result<impl Iterator<Item = DirectoryEntry> + '_, Error> {
        Ok(DirectoryIterator::new(self.root_dir_raw_iter()?))
    }

    fn allocate_clusters(&self, n: usize) -> Result<u32, Error> {
        let mut iter = ClusterAllocator::new(self, 3, ClusterAllocatorMode::CheckFree).unwrap();
        let first = iter.prev_free_cluster;
        if let Err(e) = iter.advance_by(n) {
            return Err(Error::OutOfSpace);
        }
        let mut iter = ClusterAllocator::new(self, 3, ClusterAllocatorMode::Allocate).unwrap();
        if let Err(e) = iter.advance_by(n) {
            return Err(Error::OutOfSpace);
        }
        Ok(first)
    }

    fn create_directory_entry(&self, dir: impl Iterator<Item = DirectoryRawEntry>, ent: DirectoryEntry) -> Result<(), Error> {

        let name_entries_needed = (ent.name.len() + 13) / 13;
        let mut dir_entries = Vec::new();
        dir_entries.push(DirectoryRawEntry::FileDirectoryEntry(ent.metadata));
        let mut check_sum = num::Wrapping(0_u8);
        for b in ent.metadata.file_name {
            check_sum = (check_sum << 7) + (check_sum >> 1) + num::Wrapping(b);
        }
        let mut uniname = ent.name.encode_utf16().collect::<Vec<u16>>();
        uniname.push(0);
        for i in (0..ent.name.len()).step_by(13) {
            let ord = if i >= ent.name.len() / 13 * 13 {
                0x40 | (i / 13 + 1) as u8
            } else {
                (i / 13 + 1) as u8
            };
            let name_part = if i + 13 >= uniname.len() {
                &uniname[i..]
            } else {
                &uniname[i..i + 13]
            };
            dir_entries.push(DirectoryRawEntry::LongFileNameEntry(LongFileNameEntry::new(name_part, ord, check_sum.0)));
        }
        dir_entries.reverse();

        let mut slot = Vec::new();
        let mut streak = 0;
        for dir_ent in dir {
            if let DirectoryRawEntry::UnusedEntry(sec, off) | DirectoryRawEntry::FreeEntry(sec, off) = dir_ent {
                slot.push((sec, off));
                streak += 1;
                if streak == name_entries_needed + 1 {
                    break;
                }
            } else {
                streak = 0;
                slot.clear();
            }
        }
        if slot.len() < name_entries_needed + 1 {
            return Err(Error::OutOfSpace);
        }

        let mut buf = vec![0; self.sector_size as usize];
        for ((sec, off), ent) in slot.into_iter().zip(dir_entries.into_iter()) {
            self.drive.borrow_mut().read_block(sec as u64, buf.as_mut_ptr())?;
            match ent {
                DirectoryRawEntry::FileDirectoryEntry(ent) => *unsafe { (buf.as_mut_ptr().offset(off as isize) as *mut FileDirectoryEntry).as_mut().unwrap() } = ent.clone(),
                DirectoryRawEntry::LongFileNameEntry(ent) => *unsafe { (buf.as_mut_ptr().offset(off as isize) as *mut LongFileNameEntry).as_mut().unwrap() } = ent.clone(),
                _ => (),
            }
            self.drive.borrow_mut().write_block(sec as u64, buf.as_mut_slice())?;
        }

        Ok(())
    }

    fn create_file(&self, name: String, size: u32) -> Result<File, Error> {
        let cluster = self.allocate_clusters((size as usize + self.cluster_size as usize - 1) / self.cluster_size as usize)?;
        let dir_ent = DirectoryEntry::new(name, cluster, size);
        self.create_directory_entry(self.root_dir_raw_iter()?, dir_ent)?;
        File::new(self, cluster, size as usize)
    }
}

impl FileSystem for FATFileSystem {
    fn volume_label(&self) -> String {
        String::from_utf8(match self.ebpb {
            eBPB::eBPB16(ebpb) => ebpb.volume_label,
            eBPB::eBPB32(ebpb) => ebpb.extension.volume_label,
        }.to_vec()).unwrap().trim().to_string()
    }

    fn open_file(&mut self, path: String) -> Result<Box<dyn RandomReadWrite>, Error> {
        for part in path.split("/") {

        }
        Err(Error::InvalidData)
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