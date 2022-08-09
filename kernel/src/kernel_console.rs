use crate::*;
use console;
use dev::{self, ConsoleDevice};
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt;
use dev::hal::cpu;

lazy_static! {
    pub static ref KERNEL_CONSOLE: Mutex<dev::char::VgaTextMode> = Mutex::new(dev::char::VgaTextMode::new());
    pub static ref SERIAL_CONSOLE: Mutex<dev::char::Uart16550> = Mutex::new(dev::char::Uart16550::new(0));
}

pub fn clear_screen() {
    KERNEL_CONSOLE.lock().clear_screen();
}

pub fn set_color(foreground: console::Color, background: console::Color) {
    KERNEL_CONSOLE.lock().set_color(foreground, background);
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::kernel_console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::kernel_console::_serial_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => ($crate::serial_print!("\n"));
    ($fmt:expr) => ($crate::serial_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => ($crate::serial_print!(
        concat!($fmt, "\n"), $($arg)*));
}



pub(crate) use print;
pub(crate) use println;
pub(crate) use serial_print;
pub(crate) use serial_println;

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    cpu::atomic_no_interrupts(|| {
        KERNEL_CONSOLE.lock().write_fmt(args).expect("Kernel console device failure");
    });
}

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    use core::fmt::Write;
    cpu::atomic_no_interrupts(|| {
        SERIAL_CONSOLE.lock().write_fmt(args).expect("Serial console device failure");
    });
}

pub unsafe fn deadunlock() {
    KERNEL_CONSOLE.force_unlock();
    SERIAL_CONSOLE.force_unlock();
}