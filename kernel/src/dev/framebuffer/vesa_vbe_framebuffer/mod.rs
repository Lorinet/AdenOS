use crate::*;
use dev::framebuffer::*;
use dev::Device;
use core::ptr;

pub struct VesaVbeFramebuffer {
    buffer: &'static mut [u8],
    width: usize,
    height: usize,
    pixel_format: PixelFormat,
    bytes_per_pixel: usize,
    line_length: usize,
}

impl VesaVbeFramebuffer {
    pub fn new(buffer: &'static mut [u8], width: usize, height: usize, pixel_format: PixelFormat, bytes_per_pixel: usize, line_length: usize) -> VesaVbeFramebuffer {
        VesaVbeFramebuffer {
            buffer,
            width,
            height,
            pixel_format,
            bytes_per_pixel: bytes_per_pixel,
            line_length,
        }
    }
}

impl Device for VesaVbeFramebuffer {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        // the bootloader has already initialized the framebuffer
        Ok(())
    }

    fn deinit_device(&mut self) -> Result<(), dev::Error> {
        // no need for this yet
        Ok(())
    }

    fn device_name(&self) -> &str { "Framebuffer/VesaVbeFramebuffer" }
}

impl Framebuffer for VesaVbeFramebuffer {
    #[inline(always)]
    fn get_bytes_per_pixel(&self) -> usize { self.bytes_per_pixel }

    #[inline(always)]
    fn get_pixel_format(&self) -> PixelFormat { self.pixel_format }

    #[inline(always)]
    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    #[inline(always)]
    fn clear_screen(&mut self, color: Color) {
        let pixel_data = color.pixel_data(self.pixel_format);
        let mut pixel_offset: usize;
        let mut byte_offset: usize;
        for y in 0..self.height {
            for x in 0..self.width {
                pixel_offset = y * self.line_length + x;
                byte_offset = pixel_offset * self.bytes_per_pixel;
                self.buffer[byte_offset..(byte_offset + self.bytes_per_pixel)]
                .copy_from_slice(&pixel_data[..self.bytes_per_pixel]);
            }
        }
    }

    #[inline(always)]
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let pixel_offset = y * self.line_length + x;
        let pixel_data = color.pixel_data(self.pixel_format);
        let byte_offset = pixel_offset * self.bytes_per_pixel;
        self.buffer[byte_offset..(byte_offset + self.bytes_per_pixel)]
            .copy_from_slice(&pixel_data[..self.bytes_per_pixel]);
    }
}