use crate::{*, dev::hal::mem::page_mapper};
use dev::framebuffer::*;
use alloc::{vec, vec::Vec};
use dev::Device;

pub struct VesaVbeFramebuffer {
    buffer: &'static mut [u8],
    ram_buffer: Option<Vec<u8>>,
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
            ram_buffer: None,
            width,
            height,
            pixel_format,
            bytes_per_pixel: bytes_per_pixel,
            line_length,
        }
    }

    #[inline(always)]
    fn _recursive_fill(buffer: &mut [u8], start_index: usize, end_index: usize, color: Color) {
        // very smart algorithm!
        let pixel_data = color.pixel_data();
        buffer[start_index..(start_index + 4)].copy_from_slice(pixel_data);
        let mut current_length = 4;
        let mut current_index = 4 + start_index;
        while current_index + current_length < end_index {
            buffer.copy_within(start_index..current_length + start_index, current_index);
            current_index += current_length;
            current_length *= 2;
        }
        buffer.copy_within(start_index..(end_index - current_index + start_index), current_index);
    }
}

impl Device for VesaVbeFramebuffer {
    fn init_device(&mut self) -> Result<(), dev::Error> {
        let size = self.line_length * self.height * self.bytes_per_pixel;
        self.ram_buffer = Some(vec![0; size]);
        self.ram_buffer.as_mut().unwrap().copy_from_slice(self.buffer);
        //page_mapper::set_write_combining(self.ram_buffer.as_ref().unwrap().as_ptr(), size);
        page_mapper::set_write_combining(self.buffer.as_ptr(), size);
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
        let buffer = match &mut self.ram_buffer {
            Some(buffer) => &mut buffer[..],
            None => self.buffer,
        };
        Self::_recursive_fill(buffer, 0, buffer.len(), color);
    }

    #[inline(always)]
    fn raw_buffer(&mut self) -> &mut [u8] {
        match &mut self.ram_buffer {
            Some(buffer) => &mut buffer[..],
            None => self.buffer,
        }
    }

    #[inline(always)]
    fn get_pixel_index(&self, x: usize, y: usize) -> usize {
        self.line_length * y * 4 + x
    }

    #[inline(always)]
    fn set_pixel(&mut self, x: usize, y: usize, color: Color) {
        let pixel_offset = y * self.line_length * 4 + x * 4;
        let pixel_data = color.pixel_data();
        let buffer = match &mut self.ram_buffer {
            Some(buffer) => &mut buffer[..],
            None => self.buffer,
        };
        buffer[pixel_offset..(pixel_offset + 4)].copy_from_slice(pixel_data);
    }

    #[inline(always)]
    fn draw_filled_rectangle(&mut self, rectangle: Rectangle, color: Color) {
        let buffer = match &mut self.ram_buffer {
            Some(buffer) => &mut buffer[..],
            None => self.buffer,
        };
        let (x_start, y_start) = (rectangle.x * 4, rectangle.y);
        let (mut x_end, y_end) = rectangle.end_coordinates();
        x_end *= 4;
        let line_offset_start = self.line_length * y_start * 4;
        Self::_recursive_fill(buffer, line_offset_start + x_start, line_offset_start + x_end, color);
        let line_offset_end = self.line_length * y_end * 4;
        let adder = 4 * self.line_length + x_start;
        for y in (line_offset_start..line_offset_end).step_by(4 * self.line_length) {
            buffer.copy_within((line_offset_start + x_start)..(line_offset_start + x_end), y + adder);
        }
    }

    #[inline(always)]
    fn draw_rectangle(&mut self, rectangle: Rectangle, color: Color, thickness: usize) {
        
    }

    #[inline(always)]
    fn get_line_offset(&self, y: usize) -> usize {
        self.line_length * y * 4
    }

    fn commit(&mut self) {
        match &self.ram_buffer {
            Some(buffer) => {
                self.buffer.copy_from_slice(&buffer[..]);
            },
            _ => (),
        }
    }

    fn commit_area(&mut self, area: Rectangle) {
        match &self.ram_buffer {
            Some(buffer) => {
                let start = self.get_pixel_index(area.x, area.y);
                let end = self.get_pixel_index(area.x + area.width, area.y + area.height);
                (&mut self.buffer[start..end]).copy_from_slice(&buffer[start..end]);
            },
            _ => (),
        }
    }
}