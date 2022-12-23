use crate::*;
use os;

pub trait Write {}
pub trait Read {}

pub trait ReadOrWrite {}

#[repr(C)]
pub struct Handle<T> {
    pub id: usize,
    _type_holder: Option<T>,
}

impl<T> Handle<T> {
    pub fn new(id: usize) -> Handle<T> {
        Handle {
            id,
            _type_holder: None,
        }
    }
}

impl<T: Write> Handle<T> {
    #[inline(always)]
    pub fn write(&self, buffer: &[u8], length: usize) {
        os::write(self, buffer.as_ptr(), length);
    }

    #[inline(always)]
    pub fn write_str(&self, buffer: &str, length: usize) {
        os::write(self, buffer.as_ptr(), length);
    }

    #[inline(always)]
    pub fn write_ptr(&self, buffer: *const u8, length: usize) {
        os::write(self, buffer, length);
    }
}

impl<T: Read> Handle<T> {
    #[inline(always)]
    pub fn read(&self, buffer: &mut [u8], count: usize) {
        os::read(self, buffer.as_mut_ptr(), count);
    }
}

impl<T: ReadOrWrite> Handle<T> {
    #[inline(always)]
    pub fn seek(&self, offset: usize, relative: bool) {
        os::seek(self, offset, relative);
    }
}

pub struct Process;

pub struct Stream;
impl Read for Stream {}
impl Write for Stream {}
impl ReadOrWrite for Stream {}