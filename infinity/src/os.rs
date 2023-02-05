use crate::*;
use arch;
use handle::*;

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
pub extern "C" fn read(handle: Handle, buffer: *mut u8, count: usize) -> isize {
    arch::_system_call(SYSTEM_CALL_READ, handle as usize, buffer as usize, count, 0)
}

#[inline(always)]
pub extern "C" fn write(handle: Handle, buffer: *const u8, count: usize) -> isize {
    arch::_system_call(SYSTEM_CALL_WRITE, handle as usize, buffer as usize, count, 0)
}

#[inline(always)]
pub extern "C" fn seek(handle: Handle, offset: usize, relative: bool) -> isize {
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
pub extern "C" fn create_message_queue(name: &str, endpoint: ipc::Endpoint) -> Result<Handle, Error> {
    Error::from_code_to_u32(arch::_system_call(SYSTEM_CALL_CREATE_MESSAGE_QUEUE, name.as_ptr() as usize, endpoint.into(), 0, 0))
}

#[inline(always)]
pub extern "C" fn acquire_handle(path: &str) -> Result<Handle, Error> {
    Error::from_code_to_u32(arch::_system_call(SYSTEM_CALL_ACQUIRE_HANDLE, path.as_ptr() as usize, 0, 0, 0))
}

#[inline(always)]
pub extern "C" fn release_handle(handle: Handle) -> Result<(), Error> {
    Error::from_code_to_nothing(arch::_system_call(SYSTEM_CALL_RELEASE_HANDLE,handle as usize, 0, 0, 0))
}

#[inline(always)]
pub extern "C" fn available_messages(handle: Handle) -> Result<usize, Error> {
    Error::from_code_to_usize(arch::_system_call(SYSTEM_CALL_AVAILABLE_MESSAGES, handle as usize, 0, 0, 0))
}

#[inline(always)]
pub extern "C" fn available_message_size(handle: Handle) -> Result<usize, Error> {
    Error::from_code_to_usize(arch::_system_call(SYSTEM_CALL_AVAILABLE_MESSAGE_SIZE, handle as usize, 0, 0, 0))
}