use crate::*;
use console;
use dev::{self, Write, ConsoleDevice};
use core::fmt;

mod tests;

static WIDTH: usize = 160;
static HEIGHT: usize = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: console::Color, background: console::Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

pub struct VgaTextMode {
    buffer: *mut u8,
    offset: usize,
    color: ColorCode,
}

impl VgaTextMode {
    pub const fn new() -> VgaTextMode {
        VgaTextMode { 
            buffer: 0xb8000 as *mut u8,
            offset: 0,
            color: ColorCode::new(console::Color::LightGray, console::Color::Black),
        }
    }

    fn calc_offset(x: usize, y: usize) -> isize {
        (y * WIDTH + x) as isize
    }
}

impl dev::Device for VgaTextMode {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        self.set_color(console::Color::LightGray, console::Color::Black);
        self.clear_screen();
        Ok(())
    }
    fn device_name(&self) -> &str { "Character/VGATextMode" }
}

impl dev::Write for VgaTextMode {
    type T = u8;
    fn write_one(&mut self, val: &Self::T) -> Result<(), dev::Error> {
        match val {
            b'\n' => {
                self.offset += WIDTH - (self.offset % WIDTH) - 2;
            },
            ch => unsafe {
                *self.buffer.offset(self.offset as isize) = *val;
                *self.buffer.offset(self.offset as isize + 1) = self.color.0;
            },
        };
        if self.offset <= WIDTH * HEIGHT - 2 {
            self.offset += 2;
        } else {
            for y in 1..HEIGHT {
                for x in 0..WIDTH {
                    unsafe {
                        *self.buffer.offset(VgaTextMode::calc_offset(x, y - 1)) = *self.buffer.offset(VgaTextMode::calc_offset(x, y));
                        //*self.buffer.offset(((y - 1) * WIDTH + x) as isize) = 0;
                    }
                }
            }
            for x in 0..(WIDTH / 2) {
                unsafe {
                    *self.buffer.offset(VgaTextMode::calc_offset(x * 2, HEIGHT - 1)) = b' ';
                    *self.buffer.offset(VgaTextMode::calc_offset(x * 2 + 1, HEIGHT - 1)) = self.color.0;
                }
            }
            self.offset = WIDTH * (HEIGHT - 1);
        }
        Ok(())
    }
}

impl fmt::Write for VgaTextMode {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}

impl dev::ConsoleDevice for VgaTextMode {
    fn buffer_size(&self) -> (i32, i32) {
        (80, 25)
    }

    fn clear_screen(&mut self) {
        self.offset = 0;
        for _ in 0..(HEIGHT * WIDTH / 2) {
            unsafe {
                *self.buffer.offset(self.offset as isize) = b' ';
                *self.buffer.offset((self.offset + 1) as isize) = self.color.0;
            }
            self.offset += 2
        }
        self.offset = 0;
    }

    fn set_color(&mut self, foreground: console::Color, background: console::Color) {
        self.color = ColorCode::new(foreground, background);
    }
}

unsafe impl Send for VgaTextMode {}
