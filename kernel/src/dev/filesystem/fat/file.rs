use crate::*;
use super::*;
use alloc::{vec, format};

pub struct File<'a> {
    fat_fs: &'a FATFileSystem,
    directory_entry: DirectoryEntry,
    sec_iter: FATSectorIterator<'a>,
    sector_offset: u32,
    position: u64,
}

impl<'a> Debug for File<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("File")
        .field("sec_iter", &format!("{:?}", self.sec_iter))
        .field("start_cluster", &self.directory_entry.first_cluster())
        .field("sector_offset", &self.sector_offset)
        .field("size", &self.size())
        .finish()
    }
}


impl<'a> File<'a> {
    pub fn new(fat_fs: &'a FATFileSystem, directory_entry: DirectoryEntry) -> Result<File, Error> {
        let sec_iter = FATSectorIterator::new(fat_fs.fat_iter(directory_entry.first_cluster())?);
        Ok(File {
            fat_fs,
            sec_iter,
            directory_entry,
            sector_offset: 0,
            position: 0,
        })
    }

    fn calculate_actual_position(&mut self, allocate: bool) -> Result<(), Error> {
        let sector_index = self.position as u32 / self.fat_fs.sector_size;
        let sector_offset = self.position as u32 % self.fat_fs.sector_size;

        if self.sec_iter.sector_index_from_start < sector_index {
            if self.position >= self.size() {
                if allocate {
                    let difference_bytes = self.position - self.size();
                    self.sec_iter = FATSectorIterator::new(self.fat_fs.fat_iter(self.directory_entry.first_cluster())?);
                    self.sec_iter.advance_by((self.size() - 1) as usize / self.fat_fs.sector_size as usize).expect("This should succeed");
                    let opos = self.position;
                    self.position = self.size();
                    self.sector_offset = self.position as u32 % self.fat_fs.sector_size;
                    let end_offset = difference_bytes as usize % self.fat_fs.sector_size as usize;
                    let secs_to_allocate = difference_bytes as usize / self.fat_fs.sector_size as usize;
                    let wz = vec![0; self.fat_fs.sector_size as usize];
                    for _ in 0..secs_to_allocate as u64 {
                        self._write(wz.as_slice())?;
                    }
                    self._write(&wz[..end_offset])?;
                    self.position = opos;
                } else {
                    return Err(Error::EndOfFile);
                }
            } else if let Err(_) = self.sec_iter.advance_by((sector_index - self.sec_iter.sector_index_from_start) as usize) {
                return Err(Error::IOFailure);
            }
        } else if self.sec_iter.sector_index_from_start > sector_index {
            self.sec_iter = FATSectorIterator::new(self.fat_fs.fat_iter(self.directory_entry.first_cluster())?);
            if let Err(_) = self.sec_iter.advance_by(sector_index as usize) {
                return Err(Error::IOFailure);
            }
        }
        self.sector_offset = sector_offset;
        Ok(())
    }

    fn _write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        if self.offset() + buf.len() as u64 >= self.size() {
            self.directory_entry.update_size(self.offset() as u32 + buf.len() as u32);
            self.fat_fs.in_place_update_directory_entry(&self.directory_entry)?;
        }
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
            let mut sec = self.sec_iter.current_sector();
            if i < sectors_to_write - 1 {
                if let None = self.sec_iter.next() {
                    self.fat_fs.allocate_clusters(Some(self.sec_iter.last_good_cluster), 1)?;
                    self.sec_iter.cluster_iter.cluster = self.sec_iter.last_good_cluster;
                    self.sec_iter.cluster_iter.no_more = false;
                    self.sec_iter.cluster_iter.refresh_fat()?;
                    if let None = self.sec_iter.next() {
                        return Err(Error::IOFailure);
                    }
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
        self.position += buf.len() as u64;
        Ok(buf.len())
    }
}

impl<'a> Seek for File<'a> {
    fn seek(&mut self, position: u64) -> Result<(), Error> {
        self.position = position;
        Ok(())
    }

    fn offset(&self) -> u64 {
        self.position
    }

    fn size(&self) -> u64 {
        self.directory_entry.size() as u64
    }
}

impl<'a> Read for File<'a> {
    fn read_one(&mut self) -> Result<u8, Error> {
        self.calculate_actual_position(false)?;
        let mut buf = vec![0; self.fat_fs.sector_size as usize];
        self.fat_fs.drive.borrow_mut().read_block(self.sec_iter.current_sector() as u64, buf.as_mut_ptr())?;
        let rv = buf[self.sector_offset as usize];
        self.sector_offset += 1;
        if self.sector_offset >= self.fat_fs.sector_size {
            if let Ok(()) = self.sec_iter.advance_by(1) {
                self.sector_offset = 0;
            } else {
                self.sector_offset -= 1;
            }
        }
        Ok(rv)
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.calculate_actual_position(false)?;
        let buf = if self.offset() + buf.len() as u64 >= self.size() {
            let end = buf.len() - (self.offset() + buf.len() as u64 - self.size()) as usize;
            &mut buf[..end]
        } else {
            buf
        };
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
            
            if i == 0 {
                buf[..left_buf_off].copy_from_slice(&read_buf[left_read_buf_off..right_read_buf_off_tiny]);
            } else if i == sectors_to_read - 1 {
                buf[right_buf_off..].copy_from_slice(&read_buf[..right_read_buf_off]);
            } else {
                let idx = left_buf_off + ((i - 1) as usize * self.fat_fs.sector_size as usize);
                buf[idx..idx + self.fat_fs.sector_size as usize].copy_from_slice(&read_buf);
            }

            if i < sectors_to_read - 1 {
                if let None = self.sec_iter.next() {
                    break;
                }
            }
        }
        self.sector_offset = right_read_buf_off as u32;
        Ok(buf.len())
    }
}

impl<'a> Write for File<'a> {
    fn write_one(&mut self, val: u8) -> Result<(), Error> {
        self.calculate_actual_position(true)?;
        let mut buf = vec![0; self.fat_fs.sector_size as usize];
        if self.sec_iter.cluster_iter.cluster == 0 { // if file empty
            let clu = self.fat_fs.allocate_clusters(None, 0)?;
            self.sec_iter.cluster_iter.seek(clu)?;
            self.directory_entry.update_first_cluster(clu);
            self.directory_entry.update_size(self.directory_entry.size() + 1);
            self.fat_fs.in_place_update_directory_entry(&self.directory_entry)?;
        }
        self.fat_fs.drive.borrow_mut().read_block(self.sec_iter.current_sector() as u64, buf.as_mut_ptr())?;
        buf[self.sector_offset as usize] = val;
        self.fat_fs.drive.borrow_mut().write_block(self.sec_iter.current_sector() as u64, buf.as_mut_slice())?;
        self.sector_offset += 1;
        if self.offset() >= self.size() {
            self.directory_entry.update_size(self.directory_entry.size() + 1);
            self.fat_fs.in_place_update_directory_entry(&self.directory_entry)?;
        }
        if self.sector_offset >= self.fat_fs.sector_size {
            self.sector_offset = 0;
            if let Err(_) = self.sec_iter.advance_by(1) {
                self.fat_fs.allocate_clusters(Some(self.sec_iter.cluster_iter.cluster), 1)?;
                if let Err(_) = self.sec_iter.advance_by(1) {
                    return Err(Error::IOFailure);
                }
            }
        }
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.calculate_actual_position(true)?;
        self._write(buf)
    }
}