use crate::*;
use core::fmt;
use console::ConsoleColor;
use dev::framebuffer::*;
use dev::{self, Write, ConsoleDevice};
use lazy_static::lazy_static;
use font8x8::legacy::BASIC_LEGACY;

const CHARACTER_HEIGHT: usize = 8;
const CHARACTER_WIDTH: usize = 8;

pub struct FramebufferConsole<'a, F>
where F: Framebuffer {
    framebuffer: &'a mut F,
    background_color: Color,
    foreground_color: Color,
    x: usize,
    y: usize,
    color_palette: [Color; 16],
}

impl<'a, F> FramebufferConsole<'a, F>
where F: Framebuffer {
    pub fn new(framebuffer: &'a mut F) -> FramebufferConsole<'a, F> {
        let pixel_format = framebuffer.get_pixel_format();
        FramebufferConsole {
            framebuffer,
            background_color: Color { r: 0, g: 0, b: 0 },
            foreground_color: Color { r: 0xff, g: 0xff, b: 0xff },
            x: 0,
            y: 0,
            color_palette: [
                Color::new(pixel_format, 0x08, 0x08, 0x12), // Black
                Color::new(pixel_format, 0x00, 0x55, 0xbb), // Blue
                Color::new(pixel_format, 0x00, 0xed, 0x93), // Green
                Color::new(pixel_format, 0x00, 0xfa, 0xd3), // Cyan
                Color::new(pixel_format, 0xe3, 0x4c, 0x4c), // Red
                Color::new(pixel_format, 0xdc, 0x85, 0xf2), // Magenta
                Color::new(pixel_format, 0x79, 0x54, 0x43), // Brown
                Color::new(pixel_format, 0xbb, 0xbb, 0xdd), // LightGray
                Color::new(pixel_format, 0x68, 0x68, 0x78), // DarkGray
                Color::new(pixel_format, 0x00, 0xa6, 0xf4), // LightBlue
                Color::new(pixel_format, 0x5d, 0xf0, 0x67), // LightGreen
                Color::new(pixel_format, 0x68, 0xf5, 0xd7), // LightCyan
                Color::new(pixel_format, 0xec, 0x6b, 0x64), // LightRed
                Color::new(pixel_format, 0xff, 0x9f, 0xb8), // Pink
                Color::new(pixel_format, 0xef, 0xed, 0x63), // Yellow
                Color::new(pixel_format, 0xff, 0xff, 0xff), // White
            ],
        }
    }
    
    fn _new_line(&mut self) {
        let (width, height) = self.framebuffer.get_size();
        self.x = 0;
        if self.y + CHARACTER_HEIGHT + 1 >= height - (CHARACTER_HEIGHT + 1) {
            let first_line_offset = self.framebuffer.get_line_offset(CHARACTER_HEIGHT + 1);
            let last_line_offset = self.framebuffer.get_line_offset(height - 1);
            self.framebuffer.raw_buffer().copy_within(first_line_offset..=last_line_offset, 0);
            self.framebuffer.draw_filled_rectangle(Rectangle {
                x: 0,
                y: self.y - 1,
                width: width - 1,
                height: CHARACTER_HEIGHT + 1,
            }, self.background_color);
        } else {
            self.y += CHARACTER_HEIGHT + 1;
        }
    }
}

impl<'a, F> dev::Device for FramebufferConsole<'a, F>
where F: Framebuffer {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        Ok(())
    }
    fn device_name(&self) -> &str { "Character/VesaVbeConsole" }
}

impl<'a, F> dev::Write for FramebufferConsole<'a, F>
where F: Framebuffer {
    type T = u8;
    fn write_one(&mut self, val: Self::T) -> Result<(), dev::Error> {
        match val {
            b'\n' => {
                self._new_line();
            },
            0x08 => {
                if self.x < CHARACTER_WIDTH {
                    self.y -= CHARACTER_HEIGHT + 1;
                    self.x = self.framebuffer.get_size().0 - (CHARACTER_WIDTH);
                } else { self.x -= CHARACTER_WIDTH; }
                self.framebuffer.draw_filled_rectangle(Rectangle {
                    x: self.x,
                    y: self.y,
                    width: CHARACTER_WIDTH,
                    height: CHARACTER_HEIGHT,
                }, self.background_color);
            },
            c => {
                if self.x >= self.framebuffer.get_size().0 {
                    self._new_line();
                }
                let glyph = BASIC_LEGACY[c as usize];
                for (y, row) in glyph.iter().enumerate() {
                    for x in 0..8 {
                        if ((row >> x) & 1) == 1 {
                            self.framebuffer.set_pixel(self.x + x, self.y + y, self.foreground_color);
                            self.framebuffer.set_pixel(self.x + x, self.y + y, self.foreground_color);
                            self.framebuffer.set_pixel(self.x + x, self.y + y, self.foreground_color);
                        }
                    }
                }
                self.x += CHARACTER_WIDTH;
            }
        }
        Ok(())
    }
}

impl<'a, F> fmt::Write for FramebufferConsole<'a, F>
where F: Framebuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s.as_bytes()).unwrap();
        Ok(())
    }
}

impl<'a, F> dev::ConsoleDevice for FramebufferConsole<'a, F>
where F: Framebuffer {
    fn buffer_size(&self) -> (i32, i32) {
        (80, 25) // whatever
    }

    fn clear_screen(&mut self) {
        self.x = 0;
        self.y = 0;
        self.framebuffer.clear_screen(self.background_color);
    }

    fn set_color(&mut self, foreground: ConsoleColor, background: ConsoleColor) {
        self.foreground_color = self.color_palette[foreground as usize].clone();
        self.background_color = self.color_palette[background as usize].clone();
    }
}

unsafe impl<'a, F> Send for FramebufferConsole<'a, F>
where F: Framebuffer {}
