use crate::*;
use arch;

pub const SYSTEM_CALL_READ: usize = 0;
pub const SYSTEM_CALL_WRITE: usize = 1;
pub const SYSTEM_CALL_SEEK: usize = 2;
pub const SYSTEM_CALL_RESERVED0: usize = 3;
pub const SYSTEM_CALL_EXIT: usize = 4;
pub const SYSTEM_CALL_GET_PROCESS_ID: usize = 5;
pub const SYSTEM_CALL_CREATE_MESSAGE_QUEUE: usize = 6;
pub const SYSTEM_CALL_ACQUIRE_HANDLE: usize = 7;
pub const SYSTEM_CALL_RELEASE_HANDLE: usize = 8;
pub const SYSTEM_CALL_AVAILABLE_MESSAGES: usize = 9;
pub const SYSTEM_CALL_AVAILABLE_MESSAGE_SIZE: usize = 10;

#[repr(usize)]
pub enum IOHandle {
    Input = 0,
    Output = 1,
    Log = 2,
}

fn to_signed(val: usize) -> isize {
    isize::from_ne_bytes(val.to_ne_bytes())
}

#[inline(always)]
pub extern "C" fn read(handle: u32, buffer: *mut u8, count: usize) -> isize {
    arch::_system_call(SYSTEM_CALL_READ, handle as usize, buffer as usize, count, 0)
}

#[inline(always)]
pub extern "C" fn write(handle: u32, buffer: *const u8, count: usize) -> isize {
    arch::_system_call(SYSTEM_CALL_WRITE, handle as usize, buffer as usize, count, 0)
}

#[inline(always)]
pub extern "C" fn seek(handle: u32, offset: usize, relative: bool) -> isize {
    arch::_system_call(SYSTEM_CALL_SEEK, handle as usize, offset, relative as usize, 0)
}

#[inline(always)]
pub extern "C" fn exit() -> ! {
    arch::_system_call(SYSTEM_CALL_EXIT, 0, 0, 0, 0);
    loop {}
}

#[inline(always)]
pub extern "C" fn get_process_id() -> u32 {
    arch::_system_call(SYSTEM_CALL_GET_PROCESS_ID, 0, 0, 0, 0) as u32
}

#[inline(always)]
pub extern "C" fn create_message_queue(name: &str, endpoint: u32) -> Result<u32, Error> {
    Error::from_code_to_u32(arch::_system_call(SYSTEM_CALL_CREATE_MESSAGE_QUEUE, name.as_ptr() as usize, endpoint as usize, 0, 0) as i64)
}

#[inline(always)]
pub extern "C" fn acquire_handle(path: &str) -> Result<u32, Error> {
    Error::from_code_to_u32(arch::_system_call(SYSTEM_CALL_ACQUIRE_HANDLE, path.as_ptr() as usize, 0, 0, 0) as i64)
}

#[inline(always)]
pub extern "C" fn release_handle(handle: u32) -> Result<(), Error> {
    Error::from_code_to_nothing(arch::_system_call(SYSTEM_CALL_RELEASE_HANDLE,handle as usize, 0, 0, 0) as i64)
}

#[inline(always)]
pub extern "C" fn available_messages(handle: u32) -> Result<usize, Error> {
    Error::from_code_to_usize(arch::_system_call(SYSTEM_CALL_AVAILABLE_MESSAGES, handle as usize, 0, 0, 0) as i64)
}

#[inline(always)]
pub extern "C" fn available_message_size(handle: u32) -> Result<usize, Error> {
    Error::from_code_to_usize(arch::_system_call(SYSTEM_CALL_AVAILABLE_MESSAGE_SIZE, handle as usize, 0, 0, 0) as i64)
}