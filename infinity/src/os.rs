use crate::*;
use arch;
use crate::handle::*;

pub const SYSTEM_CALL_READ: usize = 0;
pub const SYSTEM_CALL_WRITE: usize = 1;
pub const SYSTEM_CALL_SEEK: usize = 2;
pub const SYSTEM_CALL_GET_IO_HANDLE: usize = 3;
pub const SYSTEM_CALL_EXIT: usize = 4;

#[repr(usize)]
pub enum IOHandle {
    Input = 0,
    Output = 1,
    Log = 2,
}



#[inline(always)]
pub extern "C" fn read<T: Read>(handle: &Handle<T>, buffer: &mut [u8], count: usize) {
    arch::_system_call(SYSTEM_CALL_READ, handle.id, buffer as *const [u8] as *const u8 as usize, count, 0);
}

#[inline(always)]
pub extern "C" fn write<T: Write>(handle: &Handle<T>, buffer: &[u8], count: usize) {
    arch::_system_call(SYSTEM_CALL_WRITE, handle.id, buffer as *const [u8] as *const u8 as usize, count, 0);
}

#[inline(always)]
pub extern "C" fn seek<T: ReadOrWrite>(handle: &Handle<T>, offset: usize, relative: bool) {
    arch::_system_call(SYSTEM_CALL_SEEK, handle.id, offset, relative as usize, 0);
}

#[inline(always)]
pub extern "C" fn get_io_handle(process_handle: Handle<Process>, io_handle: IOHandle) -> Handle<Stream> {
    Handle::<Stream>::new(arch::_system_call(SYSTEM_CALL_GET_IO_HANDLE, process_handle.id, io_handle as usize, 0, 0))
}

#[inline(always)]
pub extern "C" fn exit() -> ! {
    arch::_system_call(SYSTEM_CALL_EXIT, 0, 0, 0, 0);
    loop {}
}

