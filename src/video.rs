use core::{mem, convert::Infallible};
use alloc::{vec, vec::Vec};
use embedded_graphics::{prelude::*, image::{ImageRawBE, ImageRaw, Image}, pixelcolor::Rgb888};
use zinc64_core::{VideoOutput, Shared};

pub struct VideoBuffer {
    dim: (usize, usize),
    palette: [u32; 16],
    pixels: Vec<u32>,
}

impl VideoBuffer {
    pub fn new(width: u32, height: u32, palette: [u32; 16]) -> VideoBuffer {
        VideoBuffer {
            dim: (width as usize, height as usize),
            palette,
            pixels: vec![0; (width * height) as usize],
        }
    }

    pub fn get_data(&self) -> &[u32] {
        self.pixels.as_ref()
    }


    pub fn get_pitch(&self) -> usize {
        self.dim.0 * mem::size_of::<u32>()
    }
}

impl VideoOutput for VideoBuffer {
    fn get_dimension(&self) -> (usize, usize) {
        self.dim
    }

    fn reset(&mut self) {
        for pixel in self.pixels.iter_mut() {
            *pixel = 0x00;
        }
    }

    fn write(&mut self, index: usize, color: u8) {
        self.pixels[index] = self.palette[color as usize];
    }
}

pub struct VideoRenderer {
    viewport_dim: (usize, usize),
    frame_buffer: psp::embedded_graphics::Framebuffer,
    video_buffer: Shared<VideoBuffer>,
}

impl VideoRenderer {
    pub fn build(
        video_buffer: Shared<VideoBuffer>,
        viewport_dim: (usize, usize),
    ) -> Result<VideoRenderer, ()> {
        let frame_buffer = psp::embedded_graphics::Framebuffer::new();

        Ok(VideoRenderer {
            viewport_dim,
            frame_buffer,
            video_buffer,
        })
    }
    pub fn render(&mut self) -> Result<(), Infallible> {
        let raw_u8_slice: &[u8] = unsafe { core::mem::transmute::<&[u32], &[u8]>(self.video_buffer.borrow().get_data()) };
        let raw: ImageRawBE<Rgb888> = ImageRaw::new(raw_u8_slice, self.video_buffer.borrow().get_pitch() as u32);
        let image = Image::new(&raw, Point::zero());
        image.draw(&mut self.frame_buffer)
    }
}

pub struct Palette;

impl Palette {
    pub fn default() -> [u32; 16] {
        [
            0x000000, // Black
            0xffffff, // White
            0x68372b, // Red
            0x70a4b2, // Cyan
            0x6f3d86, // Purple
            0x588d43, // Green
            0x352879, // Blue
            0xb8c76f, // Yellow
            0x6f4f25, // Orange
            0x433900, // Brown
            0x9a6759, // LightRed
            0x444444, // DarkGray
            0x6c6c6c, // MediumGray
            0x9ad284, // LightGreen
            0x6c5eb5, // LightBlue
            0x959595, // LightGray
        ]
    }
}
