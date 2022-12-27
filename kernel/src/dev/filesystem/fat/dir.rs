use crate::*;
use super::*;
use modular_bitfield::{bitfield, specifiers::*};
use bitflags::bitflags;
use alloc::{vec, vec::Vec, string::ToString, format};
use core::str;

use crate::dev::RandomReadWrite;

use super::{FATType, FATFileSystem};

bitflags! {
    pub struct FileAttributes: u8 {
        const READ_ONLY = 0x01;
        const HIDDEN = 0x02;
        const SYSTEM = 0x04;
        const VOLUME_ID = 0x08;
        const DIRECTORY = 0x10;
        const ARCHIVE = 0x20;
        const LFN = Self::READ_ONLY.bits | Self::HIDDEN.bits | Self::SYSTEM.bits | Self::VOLUME_ID.bits;
    }
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FileDatestamp {
    day: B5,
    month: B4,
    year: B7,
}

impl Debug for FileDatestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{:02}-{:02}", self.year() as u32 + 1980, self.month(), self.day()).as_str())
    }
}

#[bitfield]
#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FileTimestamp {
    second: B5,
    minute: B6,
    hour: B5,
}

impl Debug for FileTimestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{:02}:{:02}:{:02}", self.hour() + 2, self.minute(), self.second() * 2).as_str())
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct FileDirectoryEntry {
    pub file_name: [u8; 11],
    pub attributes: FileAttributes,
    _reserved: u8,
    pub creation_time_millis: u8,
    pub time_created: FileTimestamp,
    pub date_created: FileDatestamp,
    pub date_accessed: FileDatestamp,
    pub cluster_high: u16,
    pub time_modified: FileTimestamp,
    pub date_modified: FileDatestamp,
    pub cluster_low: u16,
    pub size: u32,
}

impl Debug for FileDirectoryEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let size = self.size;
        f.debug_struct("FileDirectoryEntry")
        .field("file_name", &str::from_utf8(&self.file_name).unwrap())
        .field("attributes", &self.attributes)
        .field("creation_time_millis", &self.creation_time_millis)
        .field("time_created", &self.time_created)
        .field("date_created", &self.date_created)
        .field("date_accessed", &self.date_accessed)
        .field("time_modified", &self.time_modified)
        .field("date_modified", &self.date_modified)
        .field("cluster", &self.cluster())
        .field("size", &size).finish()
    }
}

impl FileDirectoryEntry {
    pub fn cluster(&self) -> u32 {
        ((self.cluster_high as u32) << 16) | self.cluster_low as u32
    }
}

impl FileDirectoryEntry {
    pub fn new(name: String, first_cluster: u32, size: u32) -> FileDirectoryEntry {
        let mut shortnm = name.clone();
        shortnm.truncate(11);
        if shortnm.len() < 11 {
            for _ in 0..11 - shortnm.len() {
                shortnm += " ";
            }
        }
        FileDirectoryEntry {
            file_name: shortnm.as_bytes().try_into().unwrap(),
            _reserved: 0,
            creation_time_millis: 69,
            time_created: FileTimestamp::new(),
            date_created: FileDatestamp::new(),
            attributes: FileAttributes::READ_ONLY,
            date_accessed: FileDatestamp::new(),
            time_modified: FileTimestamp::new(),
            date_modified: FileDatestamp::new(),
            size,
            cluster_high: (first_cluster >> 16) as u16,
            cluster_low: (first_cluster & 0xFFFF) as u16,
        }
    }
}

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct LongFileNameEntry {
    order: u8,
    name_0: [u16; 5],
    attributes: FileAttributes,
    entry_type: u8,
    checksum: u8,
    name_1: [u16; 6],
    _reserved: u16,
    name_2: [u16; 2],
}

impl LongFileNameEntry {
    pub fn new(name_part: &[u16], order: u8, checksum: u8) -> LongFileNameEntry {
        let mut name_part = name_part.clone().to_vec();
        for _ in 0..13 - name_part.len() {
            name_part.push(0xFFFF);
        }
        LongFileNameEntry {
            order,
            name_0: name_part[0..5].try_into().unwrap(),
            name_1: name_part[5..11].try_into().unwrap(),
            name_2: name_part[11..13].try_into().unwrap(),
            attributes: FileAttributes::LFN,
            checksum,
            entry_type: 0,
            _reserved: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DirectoryRawEntry {
    FileDirectoryEntry(FileDirectoryEntry),
    LongFileNameEntry(LongFileNameEntry),
    UnusedEntry(u32, u32),
    FreeEntry(u32, u32),
}

pub struct DirectoryRawIterator<'a> {
    fat_fs: &'a mut FATFileSystem,
    buffer: Vec<u8>,
    cluster: Option<u32>,
    sector: usize,
    sector_offset: usize,
}

impl<'a> DirectoryRawIterator<'a> {
    pub fn new(fat_fs: &'a FATFileSystem, cluster: Option<u32>) -> Result<DirectoryRawIterator<'a>, Error> {
        let fat_fs = unsafe { (fat_fs as *const FATFileSystem as *mut FATFileSystem).as_mut().unwrap() };
        let mut buffer = vec![0; fat_fs.sector_size as usize];
        match cluster {
            Some(cluster) => fat_fs.drive.borrow_mut().read_block(fat_fs.cluster_to_sector(cluster) as u64, buffer.as_mut_ptr())?, // if root dir is in cluster
            None => fat_fs.drive.borrow_mut().read_block(fat_fs.root_dir_sector as u64, buffer.as_mut_ptr())?,
        };
        /*let buffer = match cluster {
            Some(cluster) => {
                let mut buffer = vec![0; (fat_fs.sectors_per_cluster * fat_fs.sector_size) as usize];
                fat_fs.read_cluster(cluster, buffer.as_mut_slice())?;
                buffer
            },
            None => {
                let mut buffer = vec![0; (fat_fs.root_dir_sectors * fat_fs.sector_size) as usize];
                fat_fs.drive.borrow_mut().read_blocks(fat_fs.root_dir_sector as u64, fat_fs.root_dir_sectors as u64, buffer.as_mut_ptr())?;
                buffer
            },
        };*/
        Ok(DirectoryRawIterator {
            fat_fs,
            buffer,
            cluster,
            sector: 0,
            sector_offset: 0,
        })
    }
}

impl<'a> Iterator for DirectoryRawIterator<'a> {
    type Item = DirectoryRawEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.sector_offset >= self.fat_fs.sector_size as usize {
            self.sector += 1;
            self.sector_offset = 0;
            match self.cluster {
                Some(cluster) => {
                    if self.sector >= self.fat_fs.sectors_per_cluster as usize {
                        self.cluster = match self.fat_fs.fat_iter(cluster).unwrap().next() {
                            Some(FATEntry::Cluster(cluster)) => Some(cluster),
                            _ => return None,
                        };
                        self.sector = 0;
                        self.fat_fs.drive.borrow_mut().read_block(self.fat_fs.cluster_to_sector(self.cluster.unwrap()) as u64, self.buffer.as_mut_ptr());
                    } else {
                        self.fat_fs.drive.borrow_mut().read_block(self.fat_fs.cluster_to_sector(self.cluster.unwrap()) as u64 + self.sector as u64, self.buffer.as_mut_ptr());
                    }
                },
                None => { // root dir in FAT12/16
                    if self.sector >= self.fat_fs.root_dir_sectors as usize + self.fat_fs.root_dir_sector as usize {
                        return None;
                    }
                    self.fat_fs.drive.borrow_mut().read_block(self.fat_fs.root_dir_sector as u64 + self.sector as u64, self.buffer.as_mut_ptr());
                }
            }
        }

        let ent = match self.buffer[self.sector_offset] {
            0 => {
                let sec = self.sector as u32 + match self.cluster {
                    Some(cluster) => self.fat_fs.cluster_to_sector(cluster),
                    None => self.fat_fs.root_dir_sector,
                };
                DirectoryRawEntry::FreeEntry(sec, self.sector_offset as u32)
            },
            0xE5 => {
                let sec = self.sector as u32 + match self.cluster {
                    Some(cluster) => self.fat_fs.cluster_to_sector(cluster),
                    None => self.fat_fs.root_dir_sector,
                };
                DirectoryRawEntry::UnusedEntry(sec, self.sector_offset as u32)
            },
            _ => {
                match self.buffer[self.sector_offset + 11] {
                    0x0F => DirectoryRawEntry::LongFileNameEntry(unsafe { (self.buffer.as_ptr().offset(self.sector_offset as isize) as *const LongFileNameEntry).as_ref().unwrap().clone() }),
                    _ => DirectoryRawEntry::FileDirectoryEntry(unsafe { (self.buffer.as_ptr().offset(self.sector_offset as isize) as *const FileDirectoryEntry).as_ref().unwrap().clone() }),
                }
            },
        };

        self.sector_offset += 32;

        Some(ent)
    }
}

#[derive(Debug)]
pub struct DirectoryEntry {
    pub name: String,
    pub first_cluster: u32,
    pub size: u32,
    pub metadata: FileDirectoryEntry,
}

impl DirectoryEntry {
    pub fn new(name: String, first_cluster: u32, size: u32) -> DirectoryEntry {
        let nm_str = name.clone();
        DirectoryEntry {
            name,
            first_cluster,
            size,
            metadata: FileDirectoryEntry::new(nm_str, first_cluster, size),
        }
    }
}

pub struct DirectoryIterator<'a> {
    raw_iter: DirectoryRawIterator<'a>,
    name_buffer: String,
}

impl<'a> DirectoryIterator<'a> {
    pub fn new(raw_iter: DirectoryRawIterator<'a>) -> DirectoryIterator {
        DirectoryIterator {
            raw_iter,
            name_buffer: String::new(),
        }
    }
}

impl<'a> Iterator for DirectoryIterator<'a> {
    type Item = DirectoryEntry;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let ent = self.raw_iter.next();
            if let Some(ent) = ent {
                if let DirectoryRawEntry::LongFileNameEntry(ent) = ent {
                    serial_println!("LFN: {:x?}", ent);
                    let nm0 = ent.name_0;
                    let nm1 = ent.name_1;
                    let nm2 = ent.name_2;
                    self.name_buffer.insert_str(0, String::from_utf16(&nm2).unwrap().replace("\u{FFFF}", "").as_str());
                    self.name_buffer.insert_str(0, String::from_utf16(&nm1).unwrap().replace("\u{FFFF}", "").as_str());
                    self.name_buffer.insert_str(0, String::from_utf16(&nm0).unwrap().replace("\u{FFFF}", "").as_str());
                } else if let DirectoryRawEntry::FileDirectoryEntry(ent) = ent {
                    serial_println!("SFN: {:x?}", ent);
                    let dir_ent = DirectoryEntry {
                        name: if self.name_buffer.len() > 0 {
                            self.name_buffer.trim().to_string()
                        } else {
                            let mut nm = String::from_utf8(ent.file_name.to_vec()).unwrap().trim().to_string();
                            if nm.len() > 3 {
                                nm.insert(nm.len() - 3, '.');
                            }
                            nm
                        },
                        first_cluster: ent.cluster(),
                        size: ent.size,
                        metadata: ent,
                    };
                    self.name_buffer = String::new();
                    return Some(dir_ent);
                } else if let DirectoryRawEntry::FreeEntry(_, _) = ent {
                    return None;
                }
            } else {
                return None;
            }
        }
    }
}

pub struct File<'a> {
    fat_fs: &'a FATFileSystem,
    sec_iter: FATSectorIterator<'a>,
    start_cluster: u32,
    sector_offset: u32,
    size: usize,
}

impl<'a> Debug for File<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File")
        .field("sec_iter", &format!("{:?}", self.sec_iter))
        .field("start_cluster", &self.start_cluster)
        .field("sector_offset", &self.sector_offset)
        .field("size", &self.size)
        .finish()
    }
}

impl<'a> File<'a> {
    pub fn new(fat_fs: &'a FATFileSystem, start_cluster: u32, size: usize) -> Result<File, Error> {
        let sec_iter = FATSectorIterator::new(fat_fs.fat_iter(start_cluster)?);
        Ok(File {
            fat_fs,
            sec_iter,
            start_cluster,
            sector_offset: 0,
            size,
        })
    }
}

impl<'a> Seek for File<'a> {
    fn seek(&mut self, position: u64) {
        let sector_index = position as u32 / self.fat_fs.sector_size;
        let sector_offset = position as u32 % self.fat_fs.sector_size;
        if self.sec_iter.sector_index_from_start < sector_index {
            self.sec_iter.advance_by((sector_index - self.sec_iter.sector_index_from_start) as usize);
        } else if self.sec_iter.sector_index_from_start > sector_index {
            self.sec_iter = FATSectorIterator::new(self.fat_fs.fat_iter(self.start_cluster).unwrap());
            self.sec_iter.advance_by(sector_index as usize);
        }
        self.sector_offset = sector_offset;
    }

    fn offset(&self) -> u64 {
        (self.sec_iter.sector_index_from_start * self.fat_fs.sector_size) as u64
    }

    fn seek_begin(&mut self) {
        self.seek(0)
    }

    fn seek_end(&mut self) {
        self.seek(self.size as u64 - 1)
    }
}

impl<'a> Read for File<'a> {
    fn read_one(&mut self) -> Result<u8, Error> {
        let mut buf = vec![0; self.fat_fs.sector_size as usize];
        self.fat_fs.drive.borrow_mut().read_block(self.sec_iter.current_sector() as u64, buf.as_mut_ptr())?;
        let rv = buf[self.sector_offset as usize];
        self.sector_offset += 1;
        if self.sector_offset >= self.fat_fs.sector_size {
            self.sector_offset = 0;
            self.sec_iter.advance_by(1);
        }
        Ok(rv)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        let sectors_to_read = (buf.len() as u32 + self.sector_offset + self.fat_fs.sector_size - 1) / self.fat_fs.sector_size;
        let mut read_buf = vec![0; self.fat_fs.sector_size as usize];
        let mut left_buf_off = self.fat_fs.sector_size as usize - self.sector_offset as usize;
        let mut right_read_buf_off_tiny = self.fat_fs.sector_size as usize;
        if left_buf_off > buf.len() {
            left_buf_off = buf.len();
            right_read_buf_off_tiny = self.sector_offset as usize + left_buf_off;
        }
        let right_buf_off = left_buf_off + (((buf.len() - left_buf_off) / self.fat_fs.sector_size as usize) * self.fat_fs.sector_size as usize);
        let left_read_buf_off = self.sector_offset as usize;
        let right_read_buf_off = (buf.len() + left_read_buf_off) % self.fat_fs.sector_size as usize;
        for i in 0..sectors_to_read {
            let sec = self.sec_iter.current_sector();
            self.fat_fs.drive.borrow_mut().read_block(sec as u64, read_buf.as_mut_ptr())?;
            if i < sectors_to_read - 1 {
                if let None = self.sec_iter.next() {
                    break;
                }
            }
            if i == 0 {
                buf[..left_buf_off].copy_from_slice(&read_buf[left_read_buf_off..right_read_buf_off_tiny]);
            } else if i == sectors_to_read - 1 {
                buf[right_buf_off..].copy_from_slice(&read_buf[..right_read_buf_off]);
            } else {
                let idx = left_buf_off + ((i - 1) as usize * self.fat_fs.sector_size as usize);
                buf[idx..idx + self.fat_fs.sector_size as usize].copy_from_slice(&read_buf);
            }
        }
        self.sector_offset = right_read_buf_off as u32;
        Ok(buf.len())
    }
}

impl<'a> Write for File<'a> {
    fn write_one(&mut self, val: u8) -> Result<(), Error> {
        let mut buf = vec![0; self.fat_fs.sector_size as usize];
        self.fat_fs.drive.borrow_mut().read_block(self.sec_iter.current_sector() as u64, buf.as_mut_ptr())?;
        buf[self.sector_offset as usize] = val;
        self.fat_fs.drive.borrow_mut().write_block(self.sec_iter.current_sector() as u64, buf.as_mut_slice())?;
        self.sector_offset += 1;
        if self.sector_offset >= self.fat_fs.sector_size {
            self.sector_offset = 0;
            self.sec_iter.advance_by(1);
        }

        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        let sectors_to_write = (buf.len() as u32 + self.sector_offset + self.fat_fs.sector_size - 1) / self.fat_fs.sector_size;
        let mut write_buf = vec![0; self.fat_fs.sector_size as usize];
        let mut left_buf_off = self.fat_fs.sector_size as usize - self.sector_offset as usize;
        let mut right_write_buf_off_tiny = self.fat_fs.sector_size as usize;
        if left_buf_off > buf.len() {
            left_buf_off = buf.len();
            right_write_buf_off_tiny = self.sector_offset as usize + left_buf_off;
        }
        let right_buf_off = left_buf_off + (((buf.len() - left_buf_off) / self.fat_fs.sector_size as usize) * self.fat_fs.sector_size as usize);
        let left_write_buf_off = self.sector_offset as usize;
        let right_write_buf_off = (buf.len() + left_write_buf_off) % self.fat_fs.sector_size as usize;
        for i in 0..sectors_to_write {
            let sec = self.sec_iter.current_sector();
            if i < sectors_to_write - 1 {
                if let None = self.sec_iter.next() {
                    break;
                }
            }
            if i == 0 {
                self.fat_fs.drive.borrow_mut().read_block(sec as u64, write_buf.as_mut_ptr())?;
                write_buf[left_write_buf_off..right_write_buf_off_tiny].copy_from_slice(&buf[..left_buf_off]);
            } else if i == sectors_to_write - 1 {
                self.fat_fs.drive.borrow_mut().read_block(sec as u64, write_buf.as_mut_ptr())?;
                write_buf[..right_write_buf_off].copy_from_slice(&buf[right_buf_off..]);
            } else {
                let idx = left_buf_off + ((i - 1) as usize * self.fat_fs.sector_size as usize);
                write_buf.copy_from_slice(&buf[idx..idx + self.fat_fs.sector_size as usize]);
            }
            self.fat_fs.drive.borrow_mut().write_block(sec as u64, write_buf.as_mut_slice())?;
        }
        self.sector_offset = right_write_buf_off as u32;
        Ok(buf.len())
    }
}