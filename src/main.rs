#![no_std]
#![no_main]

use sound::SoundBuffer;
use video::{Palette, VideoBuffer, VideoRenderer};
use zinc64_core::{SystemModel, new_shared, SoundOutput};
use zinc64_emu::system::{Config, C64Factory, C64};
use alloc::{boxed::Box, rc::Rc, sync::Arc};

mod sound;
mod video;
mod input;

#[macro_use]
extern crate alloc;

psp::module!("sample_module", 1, 1);

const BASIC_ROM: [u8; 8192] = *include_bytes!("../zinc64/res/rom/basic.rom");
const KERNAL_ROM: [u8; 8192] = *include_bytes!("../zinc64/res/rom/kernal.rom");
const CHAR_ROM: [u8; 4096] = *include_bytes!("../zinc64/res/rom/characters.rom");

struct NullSound;
impl SoundOutput for NullSound {
    fn reset(&self) {}
    fn write(&self, _samples: &[i16]) {}
}

fn psp_main() {
    psp::enable_home_button();
    let config = Rc::new(Config::new_with_roms(SystemModel::c64_pal(), &BASIC_ROM, &CHAR_ROM, &KERNAL_ROM));
    let sound_buffer = new_shared(NullSound {});
    let video_buffer = new_shared(VideoBuffer::new(
        config.model.frame_buffer_size.0,
        config.model.frame_buffer_size.1,
        Palette::default(),
    ));
    let mut video_renderer = VideoRenderer::build(video_buffer.clone(),
            (0, 0),
            (480, 272)
    ).unwrap();
    let chip_factory = Box::new(C64Factory::new(config.clone()));
    let mut c64 = C64::build(
        config,
        &*chip_factory,
        video_buffer.clone(),
        None,
    );
    c64.reset(false);
    let mut next_keyboard_event = 0;
    loop {
        c64.run_frame();

        if c64.is_cpu_jam() {
            panic!("CPU JAM detected at 0x{:x}", c64.get_cpu().get_pc());
        }

        video_renderer.render();
        c64.reset_vsync();


        if c64.get_keyboard().has_events() && c64.get_cycles() >= next_keyboard_event
        {
            c64.get_keyboard().drain_event();
            next_keyboard_event = c64.get_cycles().wrapping_add(20000);
        }

        unsafe { psp::sys::sceDisplayWaitVblankStart(); }
    }
}
