use crate::{*, dev::Error};
use handle::Handle;
use alloc::{boxed::Box, string::String};

pub struct FileHandle {
    id: usize,
    owner: usize,
    path: String,
}

impl FileHandle {
    pub fn new(id: usize, owner: usize, path: String) -> FileHandle {
        FileHandle {
            id,
            owner,
            path,
        }
    }
}

impl Handle for FileHandle {
    fn id(&self) -> usize {
        self.id
    }

    fn owner(&self) -> usize {
        self.owner
    }
}

pub struct File {
    fs_file: Box<dyn dev::RandomReadWrite>,
    handle: usize,
}

impl File {
    pub fn open(path: String) -> Result<File, Error> {
        Err(Error::IOFailure)
    }
}

impl dev::Seek for File {
    fn offset(&self) -> u64 {
        self.fs_file.offset()
    }

    fn seek(&mut self, position: u64) -> Result<(), Error> {
        self.fs_file.seek(position)
    }

    fn seek_begin(&mut self) -> Result<(), Error> {
        self.fs_file.seek_begin()
    }

    fn seek_end(&mut self) -> Result<(), Error> {
        self.fs_file.seek_end()
    }

    fn seek_relative(&mut self, offset: i64) -> Result<(), Error> {
        self.fs_file.seek_relative(offset)
    }

    fn size(&self) -> u64 {
        self.fs_file.size()
    }
}

impl dev::Read for File {
    fn read_one(&mut self) -> Result<u8, Error> {
        self.fs_file.read_one()
    }
    
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        self.fs_file.read(buf)
    }
}

impl dev::Write for File {
    fn write_one(&mut self, val: u8) -> Result<(), Error> {
        self.fs_file.write_one(val)
    }

    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        self.fs_file.write(buf)
    }
}