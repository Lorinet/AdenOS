#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

#[cfg(feature = "kernel_mode")]
pub static mut _system_call_handler: extern "C" fn(usize, usize, usize, usize, usize) -> isize = _dummy_system_call;

#[cfg(feature = "kernel_mode")]
pub fn _system_call(_syscall: usize, _arg0: usize, _arg1: usize, _arg2: usize, _arg3: usize) -> isize {
    unsafe {
        _system_call_handler(_syscall, _arg0, _arg1, _arg2, _arg3)
    }
}

#[cfg(feature = "kernel_mode")]
extern "C" fn _dummy_system_call(_syscall: usize, _arg0: usize, _arg1: usize, _arg2: usize, _arg3: usize) -> isize {
    0
}