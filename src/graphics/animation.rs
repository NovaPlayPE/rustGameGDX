use std::cmp;

use crate::graphics::texture::TextureRegion;


pub enum PlayMode {
    Normal,
    Reversed,
    Loop,
    LoopReversed,
    LoopPingPong,
}

pub struct Animation {
    frame_duration: f32,
    animation_duration: f32,
    key_frames: Vec<TextureRegion>,
    pub play_mode: PlayMode,
}

impl Animation {
    pub fn new(frame_duration: f32, key_frames: &[TextureRegion]) -> Option<Self> {
        if frame_duration <= 0.0 {
            return None;
        }

        Some(Self {
            frame_duration,
            animation_duration: frame_duration * key_frames.len() as f32,
            key_frames: key_frames.to_vec(),
            play_mode: PlayMode::Normal,
        })
    }

    pub fn current_key_frame(&self, run_time: f32) -> &TextureRegion {
        let num_frames = self.key_frames.len() as u32;
        if num_frames == 1 {
            return &self.key_frames[0];
        }

        let frame_number = (run_time / self.frame_duration) as u32;
        let frame_number = match self.play_mode {
            PlayMode::Normal => cmp::min(num_frames - 1, frame_number),
            PlayMode::Loop => frame_number % num_frames,
            PlayMode::LoopPingPong => {
                let frame_number = frame_number % ((num_frames * 2) - 2);
                if frame_number >= num_frames {
                    num_frames - 2 - (frame_number - num_frames)
                } else {
                    frame_number
                }
            },
            PlayMode::Reversed => cmp::max(num_frames - frame_number - 1, 0),
            PlayMode::LoopReversed => {
                let frame_number = frame_number % num_frames;
                num_frames - frame_number - 1
            },
        };

        &self.key_frames[frame_number as usize]
    }

    pub fn key_frames(&self) -> &[TextureRegion] {
        self.key_frames.as_slice()
    }

    pub fn animation_duration(&self) -> f32 {
        self.animation_duration
    }
}
