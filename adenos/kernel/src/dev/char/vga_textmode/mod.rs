use crate::*;
use dev::*;
use dev::hal::port;
use alloc::{vec, vec::Vec, string::String};
use core::fmt;

static WIDTH: usize = 160;
static HEIGHT: usize = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ColorCode(u8);

impl ColorCode {
    const fn new(foreground: ConsoleColor, background: ConsoleColor) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug)]
pub struct VgaTextMode {
    buffer: *mut u8,
    offset: usize,
    color: ColorCode,
    control_port: port::Port,
    data_port: port::Port,
}

impl VgaTextMode {
    pub const fn new() -> VgaTextMode {
        VgaTextMode { 
            buffer: 0xb8000 as *mut u8,
            offset: 0,
            color: ColorCode::new(ConsoleColor::White, ConsoleColor::Black),
            control_port: port::Port::new(0x3D4),
            data_port: port::Port::new(0x3D5),
        }
    }

    fn calc_offset(x: usize, y: usize) -> isize {
        (y * WIDTH + x) as isize
    }

    fn move_cursor(&mut self) {
        self.control_port.write(&[0x0F]).unwrap();
        self.data_port.write(&[((self.offset / 2) & 0xFF) as u8]).unwrap();
        self.control_port.write(&[0x0E]).unwrap();
        self.data_port.write(&[(((self.offset / 2) >> 8) & 0xFF) as u8]).unwrap();
    }

    pub fn disable_cursor(&mut self) {
        self.control_port.write(&[0x0A]).unwrap();
        self.data_port.write(&[0x20]).unwrap();
    }

    fn write_one(&mut self, val: u8) -> Result<(), Error> {
        match val {
            b'\n' => {
                self.offset += WIDTH - (self.offset % WIDTH) - 2;
            },
            0x08 => {
                self.offset -= 2;
                self.write_one(b' ').unwrap();
                self.offset -= 4;
            },
            ch => unsafe {
                *self.buffer.offset(self.offset as isize) = ch;
                *self.buffer.offset(self.offset as isize + 1) = self.color.0;
            },
        };
        if self.offset < WIDTH * HEIGHT - 2 {
            self.offset += 2;
        } else {
            for y in 1..HEIGHT {
                for x in 0..WIDTH {
                    unsafe {
                        *self.buffer.offset(VgaTextMode::calc_offset(x, y - 1)) = *self.buffer.offset(VgaTextMode::calc_offset(x, y));
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
        self.move_cursor();
        Ok(())
    }
}

impl Device for VgaTextMode {
    fn init_device(&mut self) -> Result<(), Error> {
        self.set_color(ConsoleColor::White, ConsoleColor::Black);
        self.clear_screen();
        Ok(())
    }

    fn device_path(&self) -> Vec<String> {
        vec![String::from("Character"), String::from("VGATextMode")]
    }

    fn unwrap(&mut self) -> DeviceClass {
        DeviceClass::WriteDevice(self)
    }
}

impl Write for VgaTextMode {
    fn write(&mut self, buf: &[u8]) -> Result<usize, Error> {
        for b in buf {
            self.write_one(*b);
        }
        Ok(buf.len())
    }
}

impl fmt::Write for VgaTextMode {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}

impl ConsoleDevice for VgaTextMode {
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

    fn set_color(&mut self, foreground: ConsoleColor, background: ConsoleColor) {
        self.color = ColorCode::new(foreground, background);
    }
}

unsafe impl Send for VgaTextMode {}
