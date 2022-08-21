mod vesa_vbe_framebuffer;
use core::{ops, slice};
pub use vesa_vbe_framebuffer::VesaVbeFramebuffer;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum PixelFormat {
    RGB,
    BGR,
    Monochrome,
}

#[repr(C)]
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    #[inline(always)]
    #[allow(non_snake_case)]
    pub fn new(pixel_format: PixelFormat, R: u8, G: u8, B: u8) -> Color {
        match pixel_format {
            PixelFormat::RGB => Color { r: R, g: G, b: B },
            PixelFormat::BGR => Color { r: B, g: G, b: R },
            PixelFormat::Monochrome => Color { r: R, g: 0, b: 0 },
        }
    }

    #[inline(always)]
    pub fn pixel_data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self as *const _ as *const u8, 4) }
    }
}

impl ops::Mul<u8> for Color {
    type Output = Color;
    fn mul(self, rhs: u8) -> Self::Output {
        Color { r: self.r * rhs, g: self.g * rhs, b: self.b * rhs }
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Rectangle {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

impl Rectangle {
    pub fn end_coordinates(&self) -> (usize, usize) {
        (self.x + self.width, self.y + self.height)
    }
}

pub trait Framebuffer {
    fn get_pixel_format(&self) -> PixelFormat;
    fn get_bytes_per_pixel(&self) -> usize;
    fn get_size(&self) -> (usize, usize);
    fn get_pixel_index(&self, x: usize, y: usize) -> usize;
    fn raw_buffer(&mut self) -> &mut [u8];
    fn set_pixel(&mut self, x: usize, y: usize, color: Color);
    fn clear_screen(&mut self, color: Color);
    fn draw_rectangle(&mut self, rectangle: Rectangle, color: Color, thickness: usize);
    fn draw_filled_rectangle(&mut self, rectangle: Rectangle, color: Color);
    fn get_line_offset(&self, y: usize) -> usize;
    fn commit(&mut self);
    fn commit_area(&mut self, area: Rectangle);
}