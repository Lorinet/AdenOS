use crate::*;
use alloc::{vec, vec::Vec};

pub enum DeviceClass<'a> {
    ReadDevice(&'a mut dyn Read),
    WriteDevice(&'a mut dyn Write),
    ReadWriteDevice(&'a mut dyn ReadWrite),
    RandomReadWriteDevice(&'a mut dyn RandomReadWrite),
    Framebuffer(&'a mut dyn Framebuffer),
    BlockDevice(&'a mut dyn BlockReadWrite),
    Other,
}

pub trait Device: Resource + Debug {
    fn init_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn deinit_device(&mut self) -> Result<(), Error> { Ok(()) }
    fn device_path(&self) -> Vec<String>;
    fn unwrap(&mut self) -> DeviceClass;/* {
        DeviceClass::Other
    }*/
}

pub trait Write {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error>;
}

#[macro_export]
macro_rules! write_one {
    ($a:expr, $b:expr) => {
        $a.write(&[$b])
    }
}

pub trait Read {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error>;
}

#[macro_export]
macro_rules! read_one {
    ($a:expr) => {
        {
            let mut _buf: [u8; 1] = [0];
            $a.read(&mut _buf).and_then(|_| Ok(_buf[0]))
        }
    }
}

pub trait ReadWrite: Read + Write {}
impl<T: Read + Write> ReadWrite for T {}

pub trait Seek {
    fn seek(&mut self, position: u64) -> Result<(), Error>;
    fn offset(&self) -> u64;
    fn seek_begin(&mut self) -> Result<(), Error> {
        self.seek(0)
    }
    fn seek_end(&mut self) -> Result<(), Error> {
        self.seek(self.size())
    }
    fn seek_relative(&mut self, offset: i64) -> Result<(), Error> {
        self.seek(((self.offset() as i64) + offset) as u64)
    }
    fn size(&self) -> u64;
}

pub trait RandomRead: Seek + Read {
    fn read_from(&mut self, buf: &mut [u8], offset: u64) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.read(buf);
        self.seek(prev_offset);
        result
    }
}

pub trait RandomWrite: Seek + Write {
    fn write_to(&mut self, buf: &[u8], offset: u64) -> Result<usize, Error> {
        let prev_offset = self.offset();
        self.seek(offset);
        let result = self.write(buf);
        self.seek(prev_offset);
        result
    }
}

impl<T> RandomRead for T where T: Seek + Read {}
impl<T> RandomWrite for T where T: Seek + Write {}

pub trait RandomReadWrite: RandomRead + RandomWrite {}
impl<T> RandomReadWrite for T where T: RandomRead + RandomWrite {}

pub trait BlockRead {
    fn block_size(&self) -> usize;
    fn read_block(&mut self, block: u64, buffer: *mut u8) -> Result<(), Error>;
    fn read_blocks(&mut self, start_block: u64, count: u64, buffer: *mut u8) -> Result<(), Error>;
}

pub trait BlockWrite: BlockRead {
    fn write_block(&mut self, block: u64, buffer: &mut [u8]) -> Result<(), Error>;
    fn write_blocks(&mut self, start_block: u64, buffer: &mut [u8]) -> Result<(), Error>;
}

pub trait BlockReadWrite: RandomRead + RandomWrite + BlockRead + BlockWrite + Device {}
impl<T: RandomRead + RandomWrite + BlockRead + BlockWrite> BlockReadWrite for T {}
