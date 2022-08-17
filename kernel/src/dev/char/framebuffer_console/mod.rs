use crate::*;
use core::fmt;
use console;
use dev::framebuffer::*;
use dev::{self, Write, ConsoleDevice, hal::port};
use noto_sans_mono_bitmap::{get_bitmap, get_bitmap_width, BitmapChar, BitmapHeight, FontWeight};

const CHARACTER_HEIGHT: usize = 14;
const CHARACTER_WIDTH: usize = get_bitmap_width(FontWeight::Regular, BitmapHeight::Size14);

pub struct FramebufferConsole<'a, F>
where F: Framebuffer {
    framebuffer: &'a mut F,
    background_color: Color,
    foreground_color: Color,
    x: usize,
    y: usize,
}

impl<'a, F> FramebufferConsole<'a, F>
where F: Framebuffer {
    pub fn new(framebuffer: &'a mut F) -> FramebufferConsole<'a, F> {
        FramebufferConsole {
            framebuffer,
            background_color: Color::black(),
            foreground_color: Color::white(),
            x: 0,
            y: 0,
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
        let (width, height) = self.framebuffer.get_size();
        match val {
            b'\n' => {
                self.x = 0;
                self.y += CHARACTER_HEIGHT;
            },
            c => {
                if self.x >= width {
                    self.x = 0;
                    self.y += CHARACTER_HEIGHT;
                }
                if self.y >= (height - CHARACTER_WIDTH) {
                    self.clear_screen();
                }
                let bitmap_char = get_bitmap(c as char, FontWeight::Regular, BitmapHeight::Size14).unwrap();
                for (y, row) in bitmap_char.bitmap().iter().enumerate() {
                    for (x, bt) in row.iter().enumerate() {
                        self.framebuffer.set_pixel(self.x + x, self.y + y, Color { R: *bt, G: 0, B: 0, A: 255 });
                    }
                }
                self.x += bitmap_char.width();
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
        self.framebuffer.clear_screen(Color::black());
    }

    fn set_color(&mut self, foreground: console::Color, background: console::Color) {
        
    }
}

unsafe impl<'a, F> Send for FramebufferConsole<'a, F>
where F: Framebuffer {}
