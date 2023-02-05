use crate::*;
use super::*;
use alloc::{vec, vec::Vec};

use super::FATFileSystem;

#[derive(Copy, Clone, Debug)]
pub enum FATEntry {
    Free(u32),
    Bad,
    EndOfChain,
    Cluster(u32),
}
#[derive(Debug)]
pub struct FATIterator<'a> {
    fat_fs: &'a FATFileSystem,
    pub cluster: u32,
    pub no_more: bool,
    sector_size: usize,
    fat_buffer: Vec<u8>,
}

impl<'a> FATIterator<'a> {
    pub fn new(fat_fs: &'a FATFileSystem, cluster: u32) -> Result<FATIterator<'a>, Error> {
        let sector_size = fat_fs.drive.lock().block_size();
        let mut iter = FATIterator {
            fat_fs,
            cluster,
            no_more: false,
            sector_size,
            fat_buffer: vec![0; sector_size],
        };
        iter.fat_fs.drive.lock().read_block(iter.cluster_index_to_sector_offset(cluster).0, iter.fat_buffer.as_mut_ptr())?;
        Ok(iter)
    }

    pub fn seek(&mut self, cluster: u32) -> Result<(), Error> {
        self.cluster = cluster;
        self.fat_fs.drive.lock().read_block(self.cluster_index_to_sector_offset(cluster).0, self.fat_buffer.as_mut_ptr())
    }

    pub fn refresh_fat(&mut self) -> Result<(), Error> {
        self.fat_fs.drive.lock().read_block(self.cluster_index_to_sector_offset(self.cluster).0, self.fat_buffer.as_mut_ptr())
    }

    fn cluster_index_to_sector_offset(&self, cluster: u32) -> (u64, usize) {
        let fat_off = match self.fat_fs.fat_type {
            FATType::FAT12 => cluster as usize + (cluster as usize / 2),
            FATType::FAT16 => (cluster * 2) as usize,
            FATType::FAT32 => (cluster * 4) as usize,
        };
        let sector = self.fat_fs.first_fat_sector as usize + (fat_off / self.sector_size);
        let offset = fat_off % self.sector_size;
        (sector as u64, offset)
    }
    
    fn cluster_ok(&self, cluster: u32) -> bool {
        match self.fat_fs.fat_type {
            FATType::FAT12 => !(cluster == 0 || (cluster >= 0xFF7 && cluster <= 0xFFF)),
            FATType::FAT16 => !(cluster == 0 || (cluster >= 0xFFF7)),
            FATType::FAT32 => !(cluster == 0 || (cluster >= 0xFFFFFFF7)),
        }
    }
}

impl<'a> Iterator for FATIterator<'a> {
    type Item = FATEntry;
    fn next(&mut self) -> Option<Self::Item> {
        if self.no_more {
            return None;
        }

        let rv = self.cluster;
        if !self.cluster_ok(rv) {
            self.no_more = true;
        } else {
            let (sec, sec_off) = self.cluster_index_to_sector_offset(self.cluster);


            match self.fat_fs.fat_type {
                FATType::FAT12 => {
                    let packed_val = (self.fat_buffer[sec_off + 1] as u16) << 8 | self.fat_buffer[sec_off] as u16;
                    self.cluster = match self.cluster & 1 {
                        0 => (packed_val & 0xFFF) as u32,
                        _ => (packed_val >> 4) as u32,
                    };
                },
                FATType::FAT16 => {
                    self.cluster = (((self.fat_buffer[sec_off as usize + 1] as u16) << 8) | self.fat_buffer[sec_off as usize] as u16) as u32;
                },
                FATType::FAT32 => {
                    self.cluster = (((self.fat_buffer[sec_off as usize + 3] as u32) << 24) | ((self.fat_buffer[sec_off as usize + 2] as u32) << 16) | ((self.fat_buffer[sec_off as usize + 1] as u32) << 8) | (self.fat_buffer[sec_off as usize] as u32)) & 0x0FFFFFFF;
                }
            };

            if self.cluster_ok(self.cluster) {
                let (sec_new, _) = self.cluster_index_to_sector_offset(self.cluster);
                if sec != sec_new {
                    self.fat_fs.drive.lock().read_block(sec_new, self.fat_buffer.as_mut_ptr()).unwrap();
                }
            }
        }

        Some(match self.fat_fs.fat_type {
            FATType::FAT12 => match rv {
                0 => FATEntry::Free(self.cluster),
                0xFF8 => FATEntry::Bad,
                0xFF8..=0xFFF => FATEntry::EndOfChain,
                n => FATEntry::Cluster(n as u32),
            },
            FATType::FAT16 => match rv {
                0 => FATEntry::Free(self.cluster),
                0xFFF8 => FATEntry::Bad,
                0xFFF8..=0xFFFF => FATEntry::EndOfChain,
                n => FATEntry::Cluster(n as u32),
            },
            FATType::FAT32 => match rv {
                0 => FATEntry::Free(self.cluster),
                0x0FFFFFF7 => FATEntry::Bad,
                0x0FFFFFF8..=0x0FFFFFFF => FATEntry::EndOfChain,
                n => FATEntry::Cluster(n),
            },
            
        })
    }
}


pub struct FATSectorIterator<'a> {
    pub cluster_iter: FATIterator<'a>,
    pub last_good_cluster: u32,
    pub sectors_per_cluster: u32,
    pub sector_index: u32,
    pub sector_index_from_start: u32,
}

impl<'a> Debug for FATSectorIterator<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FATSectorIterator")
        .field("current_cluster", &self.cluster_iter.cluster)
        .field("sector_index", &self.sector_index)
        .finish()
    }
}

impl<'a> FATSectorIterator<'a> {
    pub fn new(cluster_iter: FATIterator<'a>) -> FATSectorIterator {
        let sectors_per_cluster = cluster_iter.fat_fs.sectors_per_cluster;
        FATSectorIterator {
            cluster_iter,
            last_good_cluster: 0,
            sectors_per_cluster,
            sector_index: 0,
            sector_index_from_start: 0,
        }
    }

    pub fn current_sector(&self) -> u32 {
        self.cluster_iter.fat_fs.cluster_to_sector(self.cluster_iter.cluster) + self.sector_index
    }
}

impl<'a> Iterator for FATSectorIterator<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        Some(if self.sector_index < self.sectors_per_cluster - 1 {
            self.sector_index += 1;
            self.sector_index_from_start += 1;
            self.current_sector()
        } else {
            let prev_clu = self.cluster_iter.cluster;
            self.cluster_iter.next();
            if !self.cluster_iter.cluster_ok(self.cluster_iter.cluster) {
                self.last_good_cluster = prev_clu;
                return None;
                if let Ok(c) = self.cluster_iter.fat_fs.allocate_clusters(Some(self.cluster_iter.cluster as u32), 1) {
                    match self.cluster_iter.next() {
                        Some(FATEntry::Cluster(_)) => (),
                        _ => return None,
                    }
                } else {
                    return None;
                }
            };
            self.sector_index = 0;
            self.sector_index_from_start += 1;
            self.current_sector()
        })
    }
}

#[derive(Debug)]
pub enum ClusterAllocatorMode {
    CheckFree,
    Allocate,
    Free,
}

#[derive(Debug)]
pub struct ClusterAllocator<'a> {
    fat_fs: &'a FATFileSystem,
    pub prev_free_cluster: Option<u32>,
    sector_size: usize,
    fat_sector: u64,
    mode: ClusterAllocatorMode,
    fat_buffer: Vec<u8>,
}

impl<'a> ClusterAllocator<'a> {
    pub fn new(fat_fs: &'a FATFileSystem, first_cluster: Option<u32>, mode: ClusterAllocatorMode) -> Result<ClusterAllocator<'a>, Error> {
        let sector_size = fat_fs.drive.lock().block_size();
        let mut iter = ClusterAllocator {
            fat_fs,
            prev_free_cluster: first_cluster,
            sector_size,
            fat_sector: 0,
            mode,
            fat_buffer: vec![0; sector_size],
        };
        iter.fat_sector = iter.cluster_index_to_sector_offset(match first_cluster {
            Some(c) => c,
            None => 3,
        }).0;
        iter.fat_fs.drive.lock().read_block(iter.fat_sector, iter.fat_buffer.as_mut_ptr())?;
        Ok(iter)
    }

    fn cluster_index_to_sector_offset(&self, cluster: u32) -> (u64, usize) {
        let fat_off = match self.fat_fs.fat_type {
            FATType::FAT12 => cluster as usize + (cluster as usize / 2),
            FATType::FAT16 => (cluster * 2) as usize,
            FATType::FAT32 => (cluster * 4) as usize,
        };
        let sector = self.fat_fs.first_fat_sector as usize + (fat_off / self.sector_size);
        let offset = fat_off % self.sector_size;
        (sector as u64, offset)
    }

    fn find_free_cluster(&mut self) -> Option<u32> {
        let mut i = match self.prev_free_cluster {
            Some(c) => c,
            None => 3,
        } + 1;
        let mut wraparound = false;
        loop {
            let (sec, sec_off) = self.cluster_index_to_sector_offset(i);
            if sec != self.fat_sector {
                self.fat_fs.drive.lock().read_block(sec, self.fat_buffer.as_mut_ptr()).unwrap();
                self.fat_sector = sec;
            }
            let clu = match self.fat_fs.fat_type {
                FATType::FAT12 => {
                    let packed_val = (self.fat_buffer[sec_off + 1] as u16) << 8 | self.fat_buffer[sec_off] as u16;
                    panic!("Not implemented");
                    /*match self.prev_free_cluster & 1 {
                        0 => (packed_val & 0xFFF) as u32,
                        _ => (packed_val >> 4) as u32,
                    }*/
                },
                FATType::FAT16 => (((self.fat_buffer[sec_off as usize + 1] as u16) << 8) | self.fat_buffer[sec_off as usize] as u16) as u32,
                FATType::FAT32 => (((self.fat_buffer[sec_off as usize + 3] as u32) << 24) | ((self.fat_buffer[sec_off as usize + 2] as u32) << 16) | ((self.fat_buffer[sec_off as usize + 1] as u32) << 8) | (self.fat_buffer[sec_off as usize] as u32)) & 0x0FFFFFFF,
            };
            if clu == 0 {
                return Some(i);
            }
            i += 1;
            if i >= self.fat_fs.total_clusters {
                if wraparound {
                    return None;
                } else {
                    i = 3;
                    wraparound = true;
                }
            }
        }
    }

    pub fn finish(mut self) -> Result<(), Error> {
        let (sec, sec_off) = self.cluster_index_to_sector_offset(match self.prev_free_cluster {
            Some(clu) => clu,
            None => return Err(Error::OutOfSpace),
        });
        self.fat_fs.drive.lock().read_block(sec, self.fat_buffer.as_mut_ptr())?;
        match self.fat_fs.fat_type {
            FATType::FAT12 => panic!("Not implemented"),
            FATType::FAT16 => self.fat_buffer[sec_off..sec_off + 2].copy_from_slice(&(0xFFFF as u16).to_le_bytes()),
            FATType::FAT32 => self.fat_buffer[sec_off..sec_off + 4].copy_from_slice(&(0xFFFFFFFF as u32).to_le_bytes()),
        };
        self.fat_fs.drive.lock().write_block(sec, self.fat_buffer.as_mut_slice())?;
        Ok(())
    }
}

impl<'a> Iterator for ClusterAllocator<'a> {
    type Item = u32;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(clu) = self.find_free_cluster() {
            let (sec, sec_off) = self.cluster_index_to_sector_offset(match self.prev_free_cluster {
                Some(clu) => clu,
                None => 3,
            });
            if sec != self.fat_sector {
                self.fat_fs.drive.lock().read_block(sec, self.fat_buffer.as_mut_ptr()).unwrap();
                self.fat_sector = sec;
            }
            if let Some(cl) = self.prev_free_cluster { // do not overwrite cluster entry 3 if we did not provide chain end
                match self.fat_fs.fat_type {
                    FATType::FAT12 => panic!("Not implemented"),
                    FATType::FAT16 => self.fat_buffer[sec_off..sec_off + 2].copy_from_slice(&(clu as u16).to_le_bytes()),
                    FATType::FAT32 => self.fat_buffer[sec_off..sec_off + 4].copy_from_slice(&clu.to_le_bytes()),
                };
                if let ClusterAllocatorMode::Allocate = self.mode {
                    self.fat_fs.drive.lock().write_block(sec, self.fat_buffer.as_mut_slice());
                }
            }
            self.prev_free_cluster = Some(clu);
            self.prev_free_cluster
        } else {
            None
        }
    }
}
