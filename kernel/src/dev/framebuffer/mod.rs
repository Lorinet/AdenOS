mod vesa_vbe_framebuffer;
pub use vesa_vbe_framebuffer::VesaVbeFramebuffer;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub enum PixelFormat {
    RGBA,
    BGRA,
    Monochrome,
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Color {
    pub R: u8,
    pub G: u8,
    pub B: u8,
    pub A: u8,
}

impl Color {
    #[inline(always)]
    pub fn pixel_data(&self, pixel_format: PixelFormat) -> [u8; 4] {
        match pixel_format {
            PixelFormat::RGBA => [self.R, self.G, self.B, self.A],
            PixelFormat::BGRA => [self.B, self.G, self.R, self.A],
            PixelFormat::Monochrome => [self.R, 0, 0, 0],
        }
    }

    #[inline(always)]
    pub fn black() -> Color { Color { R: 0, G: 0, B: 0, A: 0, } }

    #[inline(always)]
    pub fn white() -> Color { Color { R: 255, G: 255, B: 255, A: 255, } }
}

pub trait Framebuffer {
    fn get_pixel_format(&self) -> PixelFormat;
    fn get_bytes_per_pixel(&self) -> usize;
    fn get_size(&self) -> (usize, usize);
    fn set_pixel(&mut self, x: usize, y: usize, color: Color);
    fn clear_screen(&mut self, color: Color);
}