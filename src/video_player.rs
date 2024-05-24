use atomic::Atomic;
use std::sync::{Arc, Mutex};
use wgpu::Texture;

#[derive(Clone, Copy)]
pub enum MediaPlayerState {
    Stopped,
    EndOfFile,
    Seeking(bool),
    Paused,
    Playing,
    Restarting,
}

pub struct VideoStreamer {}

pub struct AudioStreamer {}

pub struct Shared<T: Copy> {
    _raw_value: Arc<Atomic<T>>,
}

pub struct MediaPlayer {
    pub video_streamer: Arc<Mutex<VideoStreamer>>,
    pub audio_streamer: Option<Arc<Mutex<AudioStreamer>>>,
    pub media_player_state: Shared<MediaPlayerState>,
    pub texture_handle: Texture,
}
