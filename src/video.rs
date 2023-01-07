use core::{mem, convert::Infallible};
use alloc::{vec, vec::Vec};
use embedded_graphics::{prelude::*, image::{ImageRawBE, ImageRaw, Image, ImageRawLE}, pixelcolor::Rgb888};
use zinc64_core::{VideoOutput, Shared};

pub struct VideoBuffer {
    size: (usize, usize),
    palette: [u32; 16],
    pixels: Vec<u32>,
}

impl VideoBuffer {
    pub fn new(width: u32, height: u32, palette: [u32; 16]) -> VideoBuffer {
        VideoBuffer {
            size: (width as usize, height as usize),
            palette,
            pixels: vec![0u32; (width * height) as usize],
        }
    }

    pub fn get_data(&self) -> &[u32] {
        self.pixels.as_ref()
    }


    pub fn get_pitch(&self) -> usize {
        self.size.0 * mem::size_of::<u32>()
    }
}

impl VideoOutput for VideoBuffer {
    fn get_dimension(&self) -> (usize, usize) {
        self.size
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

pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Rect {
            x,
            y,
            w: width,
            h: height,
        }
    }

    pub fn new_with_origin(origin: (u32, u32), size: (u32, u32)) -> Self {
        Self::new(origin.0, origin.1, size.0, size.1)
    }
}

pub struct VideoRenderer {
    viewport_rect: Rect,
    frame_buffer: psp::embedded_graphics::Framebuffer,
    video_buffer: Shared<VideoBuffer>,
}

impl VideoRenderer {
    pub fn build(
        video_buffer: Shared<VideoBuffer>,
        viewport_offset: (u32, u32),
        viewport_size: (u32, u32),
    ) -> Result<VideoRenderer, ()> {
        let frame_buffer = psp::embedded_graphics::Framebuffer::new();
        let video_buffer = video_buffer.clone();
        let viewport_rect = Rect::new_with_origin(viewport_offset, viewport_size);

        Ok(VideoRenderer {
            viewport_rect,
            frame_buffer,
            video_buffer,
        })
    }
    pub fn render(&mut self) -> Result<(), Infallible> {
        let buf = self.video_buffer.borrow(); 
        let data = buf.get_data();
        let mut raw_u8_vec = Vec::new();
        for word in data {
           for (i, byte) in word.to_le_bytes().iter().enumerate() {
               if i != 3
               {
                    raw_u8_vec.push(*byte);
               }
           }
        }
        let raw: ImageRawLE<Rgb888> = ImageRaw::new(raw_u8_vec.as_slice(), buf.get_pitch() as u32/4);
        let image = Image::new(&raw, Point::new(self.viewport_rect.x as i32, self.viewport_rect.y as i32));
        image.draw(&mut self.frame_buffer);
        Ok(())
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
