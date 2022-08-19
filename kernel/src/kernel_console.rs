use crate::*;
use console::ConsoleColor;
use dev::{self, ConsoleDevice, framebuffer::{VesaVbeFramebuffer, PixelFormat}, char::{FramebufferConsole, Uart16550}};
use lazy_static::lazy_static;
use spin::Mutex;
use core::fmt::{self, Write};
use dev::hal::cpu;

pub static mut FRAMEBUFFER: Option<VesaVbeFramebuffer> = None;
pub static mut KERNEL_CONSOLE: Option<FramebufferConsole<VesaVbeFramebuffer>> = None;
pub static mut SERIAL_CONSOLE: Uart16550 = Uart16550::new(0);

pub fn clear_screen() {
    unsafe {
        KERNEL_CONSOLE.as_mut().unwrap().clear_screen();
    }
}

pub fn set_color(foreground: ConsoleColor, background: ConsoleColor) {
    unsafe {
        KERNEL_CONSOLE.as_mut().unwrap().set_color(foreground, background);
    }
}

#[macro_export]
macro_rules! early_print {
    ($($arg:tt)*) => ($crate::kernel_console::_early_print(format_args!($($arg)*)));
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
macro_rules! no_preempt_print {
    ($($arg:tt)*) => ($crate::kernel_console::_no_preempt_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! no_preempt_println {
    () => ($crate::no_preempt_print!("\n"));
    ($($arg:tt)*) => ($crate::no_preempt_print!("{}\n", format_args!($($arg)*)));
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
    unsafe {
        cpu::atomic_no_interrupts(|| {
            KERNEL_CONSOLE.as_mut().unwrap().write_fmt(args).expect("Kernel console device failure");
        });
    }
}

#[doc(hidden)]
pub fn _no_preempt_print(args: fmt::Arguments) {
    unsafe {
        cpu::atomic_no_preempt(|| {
            KERNEL_CONSOLE.as_mut().unwrap().write_fmt(args).expect("Kernel console device failure");
        });
    }
}

#[doc(hidden)]
pub fn _early_print(args: fmt::Arguments) {
    unsafe {
        KERNEL_CONSOLE.as_mut().unwrap().write_fmt(args).expect("Kernel console device failure");
    }
}

#[doc(hidden)]
pub fn _serial_print(args: ::core::fmt::Arguments) {
    unsafe {
        cpu::atomic_no_interrupts(|| {
            SERIAL_CONSOLE.write_fmt(args).expect("Serial console device failure");
        });
    }
}

pub unsafe fn deadunlock() {
    
}