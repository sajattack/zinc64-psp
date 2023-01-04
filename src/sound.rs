// This file is part of zinc64.
// Copyright (c) 2016-2019 Sebastian Jastrzebski. All rights reserved.
// Licensed under the GPLv3. See LICENSE file in the project root for full license text.

#![cfg_attr(feature = "cargo-clippy", allow(clippy::cast_lossless))]

use alloc::rc::Rc;
use alloc::sync::Arc;
use core::cell::Cell;
use core::result::Result;


const SCALER_MAX: u32 = 4096;
const SCALER_SHIFT: usize = 12;
const VOLUME_MAX: u8 = 100;

pub struct AudioCallback(pub Rc<AudioRenderer>);


pub struct AudioEngine {
    renderer: Rc<AudioRenderer>,
}

impl AudioEngine {
    pub fn build(
        freq: u32,
        samples: usize,
        buffer: Arc<SoundBuffer>,
    ) -> Result<AudioEngine, ()> {
        let renderer = Rc::new(AudioRenderer::new(buffer));
        Ok(AudioEngine { renderer })
    }

    pub fn make_irq_handler(&self) {
    }

    pub fn renderer(&self) -> &Rc<AudioRenderer> {
        &self.renderer
    }

    pub fn start(&self) {
    }
}

pub struct AudioRenderer {
    // Resources
    buffer: Arc<SoundBuffer>,
    // Runtime State
    mute: bool,
    scaler: u32,
    volume: u8,
    #[allow(unused)]
    pos: Cell<usize>,
}

#[allow(unused)]
impl AudioRenderer {
    pub fn new(buffer: Arc<SoundBuffer>) -> Self {
        let mut renderer = AudioRenderer {
            buffer,
            mute: false,
            scaler: 0,
            volume: 0,
            pos: Cell::new(0),
        };
        renderer.set_volume(VOLUME_MAX);
        renderer
    }

    pub fn is_mute(&self) -> bool {
        self.mute
    }

    pub fn set_volume(&mut self, volume: u8) {
        self.scaler = (volume as u32 * SCALER_MAX) / VOLUME_MAX as u32;
        self.volume = volume;
    }

    pub fn toggle_mute(&mut self) {
        self.mute = !self.mute;
    }

    pub fn write(&self, out: &mut [u32]) {
        if !self.mute {
            self.copy(out);
        } else {
            for x in out.iter_mut() {
                *x = 0;
            }
        }
    }

    #[inline]
    fn convert_sample(&self, value: i16) -> u32 {
        let mut sample = value as i32;
        sample += 1 << 15;
        sample >>= 4; // FIXME
        sample as u32
    }

    fn copy(&self, out: &mut [u32]) {
    }
}

use zinc64_core::SoundOutput;


pub struct SoundBuffer {
}

impl SoundBuffer {
    pub fn new(length: usize) -> Self {
        //let buffer = NullLock::new(CircularBuffer::new(length));
        SoundBuffer { }
    }

    pub fn get_data(&self) {
    }
}

impl SoundOutput for SoundBuffer {
    fn reset(&self) {
    }

    fn write(&self, samples: &[i16]) {
    }
}
