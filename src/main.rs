#![no_std]
#![no_main]

use sound::SoundBuffer;
use video::{Palette, VideoBuffer};
use zinc64_core::{SystemModel, new_shared};
use zinc64_emu::system::{Config, C64Factory, C64};
use alloc::{boxed::Box, rc::Rc, sync::Arc};

mod sound;
mod video;
mod input;

extern crate alloc;

psp::module!("sample_module", 1, 1);

const BASIC_ROM: [u8; 8192] = *include_bytes!("../zinc64/res/rom/basic.rom");
const KERNAL_ROM: [u8; 8192] = *include_bytes!("../zinc64/res/rom/kernal.rom");
const CHAR_ROM: [u8; 4096] = *include_bytes!("../zinc64/res/rom/characters.rom");

fn psp_main() {
    psp::enable_home_button();
    let config = Rc::new(Config::new_with_roms(SystemModel::c64_pal(), &BASIC_ROM, &CHAR_ROM, &KERNAL_ROM));
    let sound_buffer = Arc::new(SoundBuffer::new(config.sound.buffer_size << 2));
    let video_buffer = new_shared(VideoBuffer::new(
        config.model.frame_buffer_size.0,
        config.model.frame_buffer_size.1,
        Palette::default(),
    ));
    let chip_factory = Box::new(C64Factory::new(config.clone()));
    let mut c64 = C64::build(
        config.clone(),
        &*chip_factory,
        video_buffer.clone(),
        sound_buffer.clone(),
    );
    c64.reset(true);
    loop {
        c64.run_frame();
        unsafe { psp::sys::sceDisplayWaitVblankStart(); }
    }
}
