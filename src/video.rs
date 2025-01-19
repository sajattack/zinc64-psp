use core::{mem, convert::Infallible};
use alloc::{vec, vec::Vec};
use zinc64_core::factory::VideoOutput;
use zinc64_core::util::Shared;
use psp::Align16;

pub struct VideoBuffer {
    size: (usize, usize),
    pixels: Vec<u8>,
}

impl VideoBuffer {
    pub fn new(width: u32, height: u32) -> VideoBuffer {
        VideoBuffer {
            size: (512 as usize, height as usize),
            pixels: vec![0u8; (512 * height) as usize],
        }
    }

    pub fn get_data(&self) -> &[u8] {
        self.pixels.as_ref()
    }


    pub fn get_pitch(&self) -> usize {
        self.size.0
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
        self.pixels[index] = color;
    }
}

pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

static VERTICES: Align16<[Vertex; 2]> = Align16([
    Vertex {
        u: 0.0,
        v: 0.0,
        color: 0xffff_ffff,
        x: -40.0,
        y: 0.0,
        z: 0.0,
    },
    Vertex {
        u: 480.0,
        v: 272.0,
        color: 0xffff_ffff,
        x: 440.0,
        y: 272.0,
        z: 0.0,
    }
]);

static mut LIST: Align16<[u32; 0x40000]> = Align16([0; 0x40000]);

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
    allocator: psp::vram_alloc::SimpleVramAllocator,
    video_buffer: Shared<VideoBuffer>,
}

impl VideoRenderer {
    pub fn build(
        video_buffer: Shared<VideoBuffer>,
        viewport_offset: (u32, u32),
        viewport_size: (u32, u32),
    ) -> Result<VideoRenderer, ()> {

        let allocator = psp::vram_alloc::get_vram_allocator().unwrap();
        let video_buffer = video_buffer.clone();
        let viewport_rect = Rect::new_with_origin(viewport_offset, viewport_size);

        Ok(VideoRenderer {
            viewport_rect,
            allocator,
            video_buffer,
        })
    }

    pub fn render(&mut self) -> Result<(), Infallible> {
        let buf = self.video_buffer.borrow(); 
        let data = buf.get_data();

        unsafe {
            psp::sys::sceGuStart(psp::sys::GuContextType::Direct, LIST.0.as_mut_ptr() as _);

            psp::sys::sceGuClutMode(psp::sys::ClutPixelFormat::Psm5650, 0, 0xff, 0);

            psp::sys::sceGuTexMode(psp::sys::TexturePixelFormat::PsmT8, 0, 0, 0);
            psp::sys::sceGuTexImage(psp::sys::MipmapLevel::None, 512, 272, 512, data.as_ptr() as _);
            psp::sys::sceGuTexFunc(psp::sys::TextureEffect::Replace, psp::sys::TextureColorComponent::Rgb);
            psp::sys::sceGuDrawArray(psp::sys::GuPrimitive::Sprites, psp::sys::VertexType::COLOR_8888 | psp::sys::VertexType::TEXTURE_32BITF | psp::sys::VertexType::VERTEX_32BITF | psp::sys::VertexType::TRANSFORM_2D, 2, core::ptr::null() as _, VERTICES.0.as_ptr() as _);
            psp::sys::sceGuFinish();
            psp::sys::sceGuSync(psp::sys::GuSyncMode::Finish, psp::sys::GuSyncBehavior::Wait);
            psp::sys::sceGuSwapBuffers();
        }
        
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), Infallible> {
        unsafe 
        {

            let fbp0 = self.allocator.alloc_texture_pixels(480, 272, psp::sys::TexturePixelFormat::Psm5650).as_mut_ptr_from_zero();
            let fbp1 = self.allocator.alloc_texture_pixels(480, 272, psp::sys::TexturePixelFormat::Psm5650).as_mut_ptr_from_zero();

            psp::sys::sceGuInit();

            psp::sys::sceGuStart(psp::sys::GuContextType::Direct, LIST.0.as_mut_ptr() as _);
            psp::sys::sceGuDrawBuffer(psp::sys::DisplayPixelFormat::Psm5650, fbp0 as _, 512i32);
            psp::sys::sceGuDispBuffer(480, 272i32, fbp1 as _, 512i32);
            psp::sys::sceGuOffset(2048 - (480/ 2), 2048 - (272 / 2));
            psp::sys::sceGuViewport(2048, 2048, 480i32, 272i32);
            psp::sys::sceGuScissor(0, 0, 480i32, 272i32);
            psp::sys::sceGuEnable(psp::sys::GuState::ScissorTest);
            psp::sys::sceGuEnable(psp::sys::GuState::Texture2D);
            psp::sys::sceGuClutLoad(2, CLUT.0.as_ptr() as _);

            psp::sys::sceGuFinish();
            psp::sys::sceGuSync(psp::sys::GuSyncMode::Finish, psp::sys::GuSyncBehavior::Wait);

            psp::sys::sceGuDisplay(true);
        }
        Ok(())
    }
}

#[repr(C, packed)]
struct Vertex {
    pub u: f32,
    pub v: f32,
    pub color: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

static CLUT: Align16<[u16; 16]> =
Align16([
    0x0000, // Black
    0xffff, // White
    0x29ad, // Red
    0xb52e, // Cyan
    0x81ed, // Purple
    0x446b, // Green
    0x7946, // Blue
    0x6e37, // Yellow
    0x226d, // Orange
    0x01c8, // Brown
    0x5b33, // LightRed
    0x4228, // DarkGray
    0x6b6d, // MediumGray
    0x8693, // LightGreen
    0xb2ed, // LightBlue
    0x94b2, // LightGray
]);
