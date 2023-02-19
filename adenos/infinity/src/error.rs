use crate::*;
use alloc::string::String;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[repr(isize)]
#[derive(Debug, FromPrimitive)]
pub enum Error {
    UnknownError = -1,
    InitFailure = -2,
    DeinitFailure = -3,
    InvalidDevice = -4,
    DriverNotFound = -5,
    IOFailure = -6,
    InvalidData = -7,
    InvalidSeek = -8,
    InvalidHandle = -9,
    OutOfSpace = -10,
    BufferTooSmall = -11,
    ReadFailure = -12,
    WriteFailure = -13,
    EntryNotFound = -14,
    EndOfFile = -15,
    Permissions = -16,
    AlreadyOpen = -17,
    InvalidExecutable = -18,
    NoData = -19,

}

impl Error {
    pub fn code(&self) -> isize {
        *self as isize
    }

    pub fn from_code_to_usize(code: isize) -> Result<usize, Error> {
        if code < 0 {
            if let Some(err) = FromPrimitive::from_isize(code) {
                Err(err)
            } else {
                Err(Error::UnknownError)
            }
        } else {
            Ok(code as usize)
        }
    }

    pub fn from_code_to_u32(code: isize) -> Result<u32, Error> {
        if code < 0 {
            if let Some(err) = FromPrimitive::from_isize(code) {
                Err(err)
            } else {
                Err(Error::UnknownError)
            }
        } else {
            Ok(code as u32)
        }
    }

    pub fn from_code_to_nothing(code: isize) -> Result<(), Error> {
        if code < 0 {
            if let Some(err) = FromPrimitive::from_isize(code) {
                Err(err)
            } else {
                Err(Error::UnknownError)
            }
        } else {
            Ok(())
        }
    }
}

#[macro_export]
macro_rules! result_code_val {
    ($a:expr) => {
        match $a {
            Ok(i) => i as isize,
            Err(err) => err.code(),
        }
    }
}

#[macro_export]
macro_rules! result_code {
    ($a:expr) => {
        match $a {
            Ok(()) => 0,
            Err(err) => err.code(),
        }
    }
}