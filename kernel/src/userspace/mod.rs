use core::arch::asm;
use infinity::{os, handle::*};

pub unsafe extern "C" fn userspace_app_1() {
    asm!("nop");
    let output_handle = os::get_io_handle(Handle::new(0), os::IOHandle::Output);
    output_handle.write_str("Hello", 5);
    os::exit();
}