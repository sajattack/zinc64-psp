use core::{mem, convert::Infallible};
use alloc::{vec, vec::Vec};
//use embedded_graphics::{prelude::*, image::{ImageRawBE, ImageRaw, Image, ImageRawLE}, pixelcolor::Rgb888};
use core::ptr;
use psp::{BUF_WIDTH, SCREEN_WIDTH, SCREEN_HEIGHT};
use psp::vram_alloc::{self, VramMemChunk};
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


static mut LIST: psp::Align16<[u32; 0x40000]> = psp::Align16([0; 0x40000]);


#[repr(C, align(4))]
struct Vertex {
    u: f32,
    v: f32,
    x: f32,
    y: f32,
    z: f32
}

static VERTICES: psp::Align16<[Vertex; 2]> = psp::Align16([
    Vertex { u: 0.0, v: 0.0, x: 0.0, y: 0.0, z: 0.0},
    Vertex { u: 480.0, v: 272.0, x: 480.0, y: 272.0, z: 0.0},
]);


pub struct VideoRenderer {
    viewport_rect: Rect,
    //frame_buffer: psp::embedded_graphics::Framebuffer,
    disp:  *mut u8,
    draw: *mut u8,
    video_buffer: Shared<VideoBuffer>,
}

impl VideoRenderer {
    pub fn build(
        video_buffer: Shared<VideoBuffer>,
        viewport_offset: (u32, u32),
        viewport_size: (u32, u32),
    ) -> Result<VideoRenderer, ()> {
        //let frame_buffer = psp::embedded_graphics::Framebuffer::new();
        unsafe { 
            psp::sys::sceKernelChangeCurrentThreadAttr(0, psp::sys::ThreadAttributes::VFPU);
            psp::sys::sceDisplaySetMode(psp::sys::DisplayMode::Lcd, SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize);

            let mut allocator = vram_alloc::get_vram_allocator().unwrap();
            let disp = allocator.alloc_texture_pixels(BUF_WIDTH, SCREEN_HEIGHT, psp::sys::TexturePixelFormat::Psm8888);
            let draw = allocator.alloc_texture_pixels(BUF_WIDTH, SCREEN_HEIGHT, psp::sys::TexturePixelFormat::Psm8888);
            let tex = allocator.alloc_texture_pixels(BUF_WIDTH, SCREEN_HEIGHT, psp::sys::TexturePixelFormat::Psm8888);
            psp::sys::sceGuInit();
            psp::sys::sceGuStart(
                psp::sys::GuContextType::Direct,
                ptr::addr_of_mut!(LIST) as *mut _,
            );
            psp::sys::sceGuDrawBuffer(psp::sys::DisplayPixelFormat::Psm8888, draw.as_mut_ptr_from_zero() as _, BUF_WIDTH as i32);
            psp::sys::sceGuDispBuffer(SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32, disp.as_mut_ptr_from_zero() as _, BUF_WIDTH as i32);
            psp::sys::sceGuOffset(2048 - (SCREEN_WIDTH / 2), 2048 - (SCREEN_HEIGHT / 2));
            psp::sys::sceGuViewport(2048, 2048, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            psp::sys::sceGuScissor(0, 0, SCREEN_WIDTH as i32, SCREEN_HEIGHT as i32);
            psp::sys::sceGuEnable(psp::sys::GuState::ScissorTest);
            psp::sys::sceGuEnable(psp::sys::GuState::Texture2D);

            psp::sys::sceGumMatrixMode(psp::sys::MatrixMode::Projection);
            psp::sys::sceGumLoadIdentity();
            psp::sys::sceGumOrtho(0.0, 480.0, 272.0, 0.0, -30.0, 30.0);

            psp::sys::sceGumMatrixMode(psp::sys::MatrixMode::View);
            psp::sys::sceGumLoadIdentity();
            psp::sys::sceGumMatrixMode(psp::sys::MatrixMode::Model);
            psp::sys::sceGumLoadIdentity();

            psp::sys::sceGuTexMode(psp::sys::TexturePixelFormat::Psm8888, 0, 0, 0);
            psp::sys::sceGuTexFunc(psp::sys::TextureEffect::Replace, psp::sys::TextureColorComponent::Rgb);
            psp::sys::sceGuTexFilter(psp::sys::TextureFilter::Linear, psp::sys::TextureFilter::Linear);
            psp::sys::sceGuTexScale(1.0, 1.0);
            psp::sys::sceGuTexOffset(0.0, 0.0);
            psp::sys::sceGuTexWrap(psp::sys::GuTexWrapMode::Clamp, psp::sys::GuTexWrapMode::Clamp);

            psp::sys::sceGuFinish();
            psp::sys::sceGuSync(psp::sys::GuSyncMode::Finish, psp::sys::GuSyncBehavior::Wait);
            psp::sys::sceGuDisplay(true);

            let video_buffer = video_buffer.clone();
            let viewport_rect = Rect::new_with_origin(viewport_offset, viewport_size);

            Ok(VideoRenderer {
                viewport_rect,
                disp: disp.as_mut_ptr_direct_to_vram(),
                draw: draw.as_mut_ptr_direct_to_vram(),
                //frame_buffer,
                video_buffer,
            })
        }
    }
    //pub fn render(&mut self) -> Result<(), Infallible> {
        //let buf = self.video_buffer.borrow(); 
        //let data = buf.get_data();
        //let mut raw_u8_vec = Vec::new();
        //for word in data {
           //for (i, byte) in word.to_le_bytes().iter().enumerate() {
               //if i != 3
               //{
                    //raw_u8_vec.push(*byte);
               //}
           //}
        //}
        //let raw: ImageRawLE<Rgb888> = ImageRaw::new(raw_u8_vec.as_slice(), buf.get_pitch() as u32/4);
        //let image = Image::new(&raw, Point::new(self.viewport_rect.x as i32, self.viewport_rect.y as i32));
        //image.draw(&mut self.frame_buffer);
        //Ok(())
    //}
    
    pub fn render(&mut self) -> Result<(), Infallible> {

        let size = self.video_buffer.borrow().size;
        unsafe { 
            let data = self.video_buffer.borrow_mut().get_data().iter().map(|p| {
                    let bytes = p.to_le_bytes();
                    let ret: u32 = (bytes[0] as u32) << 16 | (bytes[1] as u32) << 8 | bytes[2] as u32;
                    ret
                }
            ).collect::<Vec<u32>>();
            psp::sys::sceGuStart(psp::sys::GuContextType::Direct, &mut LIST.0 as *mut _ as _);
            psp::sys::sceGuTexImage(psp::sys::MipmapLevel::None, size.0 as i32 , size.1 as i32, 512, data.as_ptr() as *const _);


            psp::sys::sceGumDrawArray(
                psp::sys::GuPrimitive::Sprites, 
                psp::sys::VertexType::TEXTURE_32BITF | psp::sys::VertexType::TRANSFORM_2D | psp::sys::VertexType::VERTEX_32BITF,
                2,
                core::ptr::null_mut(), 
                &VERTICES as *const _ as *const _
            );

            psp::sys::sceGuFinish();
            psp::sys::sceGuSync(psp::sys::GuSyncMode::Finish, psp::sys::GuSyncBehavior::Wait);
            psp::sys::sceGuSwapBuffers();
        }
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
