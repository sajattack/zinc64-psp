//#![no_std]
//#![no_main]

//use sound::SoundBuffer;
//use video::{VideoBuffer, VideoRenderer};
use zinc64_core::factory::{SystemModel, SoundOutput, VideoOutput};
use zinc64_system::{Config, C64Factory, C64};
use std::{boxed::Box, rc::Rc, sync::Arc};

//mod sound;
//mod video;
//mod input;

//#[macro_use]
//extern crate alloc;

//psp::module!("sample_module", 1, 1);

const BASIC_ROM: [u8; 8192] = *include_bytes!("../zinc64/res/rom/basic.rom");
const KERNAL_ROM: [u8; 8192] = *include_bytes!("../zinc64/res/rom/kernal.rom");
const CHAR_ROM: [u8; 4096] = *include_bytes!("../zinc64/res/rom/characters.rom");

struct NullSound;
impl SoundOutput for NullSound {
    fn reset(&self) {}
    fn write(&self, _samples: &[i16]) {}
}

struct NullVideo;
    impl VideoOutput for NullVideo {
        fn get_dimension(&self) -> (usize, usize) {
            (0, 0)
        }
        fn reset(&mut self) {}
        fn write(&mut self, _index: usize, _color: u8) {}
    }

#[used]
static mut CURRENT_TICK: u64 = 0;

fn main() {
    //psp::enable_home_button();
    let config = Rc::new(Config::new_with_roms(SystemModel::c64_pal(), &BASIC_ROM, &CHAR_ROM, &KERNAL_ROM));
    //let sound_buffer = new_shared(NullSound {});
    //let video_buffer = new_shared(VideoBuffer::new(
        //config.model.frame_buffer_size.0,
        //config.model.frame_buffer_size.1,
    //));
    //let mut video_renderer = VideoRenderer::build(video_buffer.clone(),
            //(0, 0),
            //(512, 272)
    //).unwrap();

    //video_renderer.init().unwrap();

    let chip_factory = Box::new(C64Factory::new(config.clone()));
    let mut c64 = C64::build(
        config,
        &*chip_factory,
        Rc::new(std::cell::RefCell::new(NullVideo)),
        Arc::new(NullSound),
    );
    c64.reset(false);
    let mut next_keyboard_event = 0;
    let mut frame_end: u64 = 0;
    let mut frame_start: u64 = 0;

    let mut render_end: u64 = 0;
    let mut render_start: u64 = 0;

    let mut emu_end: u64 = 0;
    loop {
        //unsafe { psp::sys::sceRtcGetCurrentTick(&mut frame_start as *mut u64) };
        c64.run_frame();
        //unsafe { psp::sys::sceRtcGetCurrentTick(&mut emu_end as *mut u64) };

        if c64.is_cpu_jam() {
            panic!("CPU JAM detected at 0x{:x}", c64.get_cpu().get_pc());
        }

        //unsafe { psp::sys::sceRtcGetCurrentTick(&mut render_start as *mut u64) };
        //video_renderer.render().unwrap();
        //unsafe { psp::sys::sceRtcGetCurrentTick(&mut render_end as *mut u64) };
        c64.reset_vsync();


        if c64.get_keyboard().has_events() && c64.get_cycles() >= next_keyboard_event
        {
            c64.get_keyboard().drain_event();
            next_keyboard_event = c64.get_cycles().wrapping_add(20000);
        }
        /*unsafe { psp::sys::sceRtcGetCurrentTick(&mut frame_end as *mut u64); }

        let ticks_per_sec = unsafe { psp::sys::sceRtcGetTickResolution() };
        let string = format!("iter time: {:.2} ms\n\0", ((frame_end - frame_start) as f32 / ticks_per_sec as f32 * 1000.0));
        unsafe { psp::sys::sceIoWrite(psp::sys::SceUid(1), string.as_bytes().as_ptr() as *mut core::ffi::c_void, string.len()); }

        let string = format!("render time: {:.2} ms\n\0", ((render_end - render_start) as f32 / ticks_per_sec as f32 * 1000.0));
        unsafe { psp::sys::sceIoWrite(psp::sys::SceUid(1), string.as_bytes().as_ptr() as *mut core::ffi::c_void, string.len()); }

        let string = format!("emu time: {:.2} ms\n\0", ((emu_end - frame_start) as f32 / ticks_per_sec as f32 * 1000.0));
        unsafe { psp::sys::sceIoWrite(psp::sys::SceUid(1), string.as_bytes().as_ptr() as *mut core::ffi::c_void, string.len()); }

        let string = format!("CURRENT_TICK: {}\n\0", unsafe { CURRENT_TICK });
        unsafe { psp::sys::sceIoWrite(psp::sys::SceUid(1), string.as_bytes().as_ptr() as *mut core::ffi::c_void, string.len()); }




        //unsafe { psp::sys::sceDisplayWaitVblankStart(); }
        */
    }
}
