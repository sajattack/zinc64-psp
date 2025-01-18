use core::{mem, convert::Infallible};
use alloc::{vec, vec::Vec};
use zinc64_core::{VideoOutput, Shared};
use psp::Align16;

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
        self.size.0 * 4
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

static VERTICES: Align16<[Vertex; 2]> = Align16([
    Vertex {
        u: 0.0,
        v: 0.0,
        color: 0xffff_ffff,
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    Vertex {
        u: 512.0,
        v: 272.0,
        color: 0xffff_ffff,
        x: 512.0,
        y: 272.0,
        z: 0.0,
    }
]);

static mut LIST: Align16<[u32; 0x10000]> = Align16([0; 0x10000]);

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
        let data = Align16(buf.get_data());

        unsafe {
            psp::sys::sceGuStart(psp::sys::GuContextType::Direct, LIST.0.as_mut_ptr() as _);
            psp::sys::sceGuTexMode(psp::sys::TexturePixelFormat::Psm8888, 0, 0, 0);
            psp::sys::sceGuTexImage(psp::sys::MipmapLevel::None, 512, 272, 512, data.0.as_ptr() as _);
            psp::sys::sceGuEnable(psp::sys::GuState::Texture2D);
            psp::sys::sceGuDrawArray(psp::sys::GuPrimitive::Sprites, psp::sys::VertexType::COLOR_8888 | psp::sys::VertexType::TEXTURE_32BITF | psp::sys::VertexType::VERTEX_32BITF | psp::sys::VertexType::TRANSFORM_2D, 2, core::ptr::null() as _, VERTICES.0.as_ptr() as _);
            psp::sys::sceGuDisable(psp::sys::GuState::Texture2D); // Questionably necessary?
            psp::sys::sceGuFinish();
            psp::sys::sceGuSync(psp::sys::GuSyncMode::Finish, psp::sys::GuSyncBehavior::Wait);
            psp::sys::sceGuSwapBuffers();
        }
        
        Ok(())
    }

    pub fn init(&mut self) -> Result<(), Infallible> {
        unsafe 
        {

            let fbp0 = self.allocator.alloc_texture_pixels(512, 272, psp::sys::TexturePixelFormat::Psm8888).as_mut_ptr_from_zero();
            let fbp1 = self.allocator.alloc_texture_pixels(512, 272, psp::sys::TexturePixelFormat::Psm8888).as_mut_ptr_from_zero();

            psp::sys::sceGuInit();

            psp::sys::sceGuStart(psp::sys::GuContextType::Direct, LIST.0.as_mut_ptr() as _);
            psp::sys::sceGuDrawBuffer(psp::sys::DisplayPixelFormat::Psm8888, fbp0 as _, 512i32);
            psp::sys::sceGuDispBuffer(512i32, 272i32, fbp1 as _, 512i32);
            psp::sys::sceGuOffset(2048 - (512 / 2), 2048 - (272 / 2));
            psp::sys::sceGuViewport(2048, 2048, 512i32, 272i32);
            psp::sys::sceGuScissor(0, 0, 512i32, 272i32);
            psp::sys::sceGuEnable(psp::sys::GuState::ScissorTest);
            psp::sys::sceGuEnable(psp::sys::GuState::Texture2D);

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

pub struct Palette;

impl Palette {
    pub fn default() -> [u32; 16] {
        [
            0xff000000, // Black
            0xffffffff, // White
            0xff2b3768, // Red
            0xffb2a470, // Cyan
            0xff863d6f, // Purple
            0xff438d58, // Green
            0xff792835, // Blue
            0xff6fc7b8, // Yellow
            0xff254f6f, // Orange
            0xff003943, // Brown
            0xff59679a, // LightRed
            0xff444444, // DarkGray
            0xff6c6c6c, // MediumGray
            0xff84d29a, // LightGreen
            0xffb55e6c, // LightBlue
            0xff959595, // LightGray
        ]
    }
}
