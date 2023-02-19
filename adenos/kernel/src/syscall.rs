use crate::{*, exec::scheduler, ipc::MessageQueue};
use core::{str, slice};
use alloc::{vec, vec::Vec, string::ToString, boxed::Box};
use {namespace, ipc::*};
use cstr_core::CStr;

#[no_mangle]
#[inline(always)]
#[allow(unused_variables)]
pub extern "C" fn system_call(syscall: usize, arg0: usize, arg1: usize, arg2: usize, arg3: usize) -> isize {
    //serial_println!("syscall {:x} {:x} {:x} {:x} {:x}", syscall, arg0, arg1, arg2, arg3);
    match syscall {
        SYSTEM_CALL_READ => _read(arg0, arg1 as *mut u8, arg2),
        SYSTEM_CALL_WRITE => _write(arg0, arg1 as *const u8, arg2),
        SYSTEM_CALL_SEEK => _seek(arg0, arg1 as i64, arg2 == 1),
        SYSTEM_CALL_RESERVED0 => 0,
        SYSTEM_CALL_EXIT => _exit(),
        SYSTEM_CALL_GET_PROCESS_ID => _get_process_id(),
        SYSTEM_CALL_CREATE_MESSAGE_QUEUE => _create_message_queue(c_str(arg0), arg1 as u32),
        SYSTEM_CALL_ACQUIRE_HANDLE => _acquire_handle(c_str(arg0)),
        SYSTEM_CALL_RELEASE_HANDLE => _release_handle(arg0 as u32),
        SYSTEM_CALL_MESSAGE_QUEUE_COUNT => _available_messages(arg0 as u32),
        SYSTEM_CALL_AVAILABLE_MESSAGE_SIZE => _available_message_size(arg0 as u32),
        _ => 1,
    }
}

fn c_str(addr: usize) -> &'static str {
    unsafe {
        CStr::from_ptr(addr as *const _).to_str().unwrap()
    }
}

pub fn _write(_handle: usize, _buffer: *const u8, _count: usize) -> isize {
    if let Some(hndl) = namespace::get_rw_handle(_handle as u32) {
        result_code_val!(hndl.write(unsafe { slice::from_raw_parts(_buffer, _count) }))
    } else {
        Error::InvalidHandle.code() as isize
    }
}

pub fn _read(_handle: usize, _buffer: *mut u8, _count: usize) -> isize {
    if let Some(hndl) = namespace::get_rw_handle(_handle as u32) {
        result_code_val!(hndl.read(unsafe { slice::from_raw_parts_mut(_buffer, _count) }))
    } else {
        Error::InvalidHandle.code() as isize
    }
}

pub fn _seek(_handle: usize, _offset: i64, relative: bool) -> isize {
    if let Some(hndl) = namespace::get_seek_handle(_handle as u32) {
        if relative {
            result_code!(hndl.seek_relative(_offset))
        } else {
            result_code!(hndl.seek(_offset as u64))
        }
    } else {
        Error::InvalidHandle.code() as isize
    }
}

pub fn _exit() -> isize {
    scheduler::terminate(scheduler::current_process());
    0
}

pub fn _get_process_id() -> isize {
    scheduler::current_process() as isize
}

pub fn _create_message_queue(name: &str, endpoint: u32) -> isize {
    let endpoint = endpoint.into();
    namespace::register_resource(MessageChannel::new(name.to_string(), Box::new(MessageQueue::new(scheduler::current_process(), endpoint, 128))));
    _acquire_handle(namespace::concat_resource_path(vec!["Processes".to_string(), scheduler::current_process().to_string(), "MessageQueues".to_string(), name.to_string()]).as_str())
}

pub fn _acquire_handle(resource_path: &str) -> isize {
    match namespace::acquire_handle(resource_path.to_string(), scheduler::current_process()) {
        Ok(hndl) => hndl.id as isize,
        Err(err) => err.code(),
    }
}

pub fn _release_handle(id: u32) -> isize {
    match namespace::release_handle(id) {
        Ok(()) => 0,
        Err(err) => err.code(),
    }
}

pub fn _available_messages(handle: u32) -> isize {
    match namespace::get_message_channel_handle(handle) {
        Some(que) => que.available() as isize,
        None => Error::InvalidHandle.code(),
    }
}

pub fn _available_message_size(handle: u32) -> isize {
    match namespace::get_message_channel_handle(handle) {
        Some(que) => match que.peek_len() {
            Ok(len) => len as isize,
            Err(err) => err.code(),
        },
        None => Error::InvalidHandle.code(),
    }
}