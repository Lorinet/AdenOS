use crate::*;
use dev::*;
use handle::*;
use alloc::vec::Vec;

pub enum Endpoint {
    Any,
    Process(u32),
}

impl Into<usize> for Endpoint {
    fn into(self) -> usize {
        match self {
            Endpoint::Any => 0,
            Endpoint::Process(pid) => pid as usize,
        }
    }
}

impl From<usize> for Endpoint {
    fn from(val: usize) -> Self {
        if val == 0 {
            Endpoint::Any
        } else {
            Endpoint::Process(val as u32)
        }
    }
}
impl From<u32> for Endpoint {
    fn from(val: u32) -> Self {
        if val == 0 {
            Endpoint::Any
        } else {
            Endpoint::Process(val as u32)
        }
    }
}

pub struct Channel {
    receiver: Handle,
    sender: Handle,
}

impl Channel {
    pub fn new(receiver: Handle, sender: Handle) -> Channel {
        Channel {
            receiver,
            sender,
        }
    }
}

impl Read for Channel {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Error> {
        Error::from_code_to_usize(os::read(self.receiver, buf.as_mut_ptr(), buf.len()))
    }
}

impl Write for Channel {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        Error::from_code_to_usize(os::write(self.sender, buf.as_ptr(), buf.len()))
    }
}