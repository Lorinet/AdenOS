use crate::boot_info::{FrameBufferInfo, PixelFormat};
use conquer_once::spin::OnceCell;
use core::{
    fmt::{self, Write},
    ptr,
};
use font8x8::legacy::BASIC_LEGACY;
use spin::Mutex;

const CHARACTER_HEIGHT: usize = 8;
const CHARACTER_WIDTH: usize = 8;

/// The global logger instance used for the `log` crate.
pub static mut LOGGER: Option<Mutex<Logger>> = None;

/// Allows logging text to a pixel-based framebuffer.
pub struct Logger {
    framebuffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl Logger {
    /// Creates a new logger that uses the given framebuffer.
    pub fn new(framebuffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut logger = Self {
            framebuffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        logger.clear();
        logger
    }

    fn newline(&mut self) {
        self.y_pos += 8;
        self.carriage_return()
    }

    fn carriage_return(&mut self) {
        self.x_pos = 0;
    }

    /// Erases all text on the screen.
    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        let start_index = 0;
        let end_index = self.framebuffer.len();
        let pixel_data = if let PixelFormat::RGB = self.info.pixel_format {
            &[0x10, 0x0f, 0xa5, 0x00]
        } else {
            &[0xa5, 0x0f, 0x10, 0x00]
        };
        self.framebuffer[start_index..(start_index + 4)].copy_from_slice(pixel_data);
        let mut current_length = 4;
        let mut current_index = 4 + start_index;
        while current_index + current_length < end_index {
            self.framebuffer.copy_within(start_index..current_length + start_index, current_index);
            current_index += current_length;
            current_length *= 2;
        }
        self.framebuffer.copy_within(start_index..(end_index - current_index + start_index), current_index);
    }

    fn width(&self) -> usize {
        self.info.horizontal_resolution
    }

    fn height(&self) -> usize {
        self.info.vertical_resolution
    }

    fn write_char(&mut self, val: char) {
        match val as u8 {
            b'\n' => {
                self.newline();
            },
            c => {
                if self.x_pos >= self.width() {
                    self.newline();
                }
                let mut ci = c as usize;
                if ci > 127 {
                    ci = 0x2E;
                }
                let glyph = BASIC_LEGACY[ci];
                for (y_pos, row) in glyph.iter().enumerate() {
                    for x_pos in 0..8 {
                        if ((row >> x_pos) & 1) == 1 {
                            self.set_pixel(self.x_pos + x_pos, self.y_pos + y_pos);
                            self.set_pixel(self.x_pos + x_pos, self.y_pos + y_pos);
                            self.set_pixel(self.x_pos + x_pos, self.y_pos + y_pos);
                        }
                    }
                }
                self.x_pos += CHARACTER_WIDTH;
            }
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize) {
        let idx = y * self.width() * 4 + x * 4;
        
        let c = &[0xb8, 0xb8, 0xb8, 0xff];
        self.framebuffer[idx..(idx + 4)].copy_from_slice(c);
    }
}

unsafe impl Send for Logger {}
unsafe impl Sync for Logger {}

impl fmt::Write for Logger {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    unsafe {
        LOGGER.as_mut().unwrap().lock().write_fmt(args).expect("Kernel console device failure");
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::binary::logger::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => (print!("{}\n", format_args!($($arg)*)));
}