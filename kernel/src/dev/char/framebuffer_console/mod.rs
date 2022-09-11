use crate::*;
use core::fmt;
use console::ConsoleColor;
use dev::framebuffer::*;
use dev::{self, Write};
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
                Color::new(pixel_format, 0x0d, 0x10, 0x1a), // Black n
                Color::new(pixel_format, 0xbf, 0x61, 0x6a), // Red n
                Color::new(pixel_format, 0x68, 0x87, 0x4d), // Green n
                Color::new(pixel_format, 0xf0, 0xdb, 0x54), // Yellow n
                Color::new(pixel_format, 0x81, 0xa1, 0xc1), // Blue n
                Color::new(pixel_format, 0xb4, 0x8e, 0xad), // Magenta n
                Color::new(pixel_format, 0x50, 0x85, 0x94), // Cyan n
                Color::new(pixel_format, 0xd8, 0xde, 0xe9), // White n
                Color::new(pixel_format, 0x2e, 0x34, 0x40), // BrightBlack n
                Color::new(pixel_format, 0xf5, 0x87, 0x87), // BrightRed n
                Color::new(pixel_format, 0xa3, 0xbe, 0x8c), // BrightGreen n
                Color::new(pixel_format, 0xeb, 0xc8, 0x8b), // BrightYellow n
                Color::new(pixel_format, 0xaa, 0xd2, 0xfa), // BrightBlue n
                Color::new(pixel_format, 0xb4, 0x8e, 0xad), // BrightMagenta
                Color::new(pixel_format, 0x88, 0xc0, 0xd0), // BrightCyan n
                Color::new(pixel_format, 0xff, 0xff, 0xff), // BrightWhite n
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
            self.framebuffer.commit();
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
                self.framebuffer.commit_area(Rectangle {
                    x: self.x,
                    y: self.y,
                    width: CHARACTER_WIDTH,
                    height: CHARACTER_HEIGHT,
                });
                self.x += CHARACTER_WIDTH;
            },
            c => {
                if self.x >= self.framebuffer.get_size().0 {
                    self._new_line();
                }
                let mut ci = c as usize;
                if ci > 127 {
                    ci = 0x2E;
                }
                let glyph = BASIC_LEGACY[ci];
                for (y, row) in glyph.iter().enumerate() {
                    for x in 0..8 {
                        if ((row >> x) & 1) == 1 {
                            self.framebuffer.set_pixel(self.x + x, self.y + y, self.foreground_color);
                            self.framebuffer.set_pixel(self.x + x, self.y + y, self.foreground_color);
                            self.framebuffer.set_pixel(self.x + x, self.y + y, self.foreground_color);
                        }
                    }
                }
                self.framebuffer.commit_area(Rectangle {
                    x: self.x,
                    y: self.y,
                    width: CHARACTER_WIDTH,
                    height: CHARACTER_HEIGHT,
                });
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
