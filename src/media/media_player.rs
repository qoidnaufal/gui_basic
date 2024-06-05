use ffmpeg_the_third as ffmpeg;
use std::sync::{Arc, Mutex};

use super::{pipeline, video};
use crate::{texture, uniforms::Uniforms};

const WIDTH_TO_HEIGHT_RATIO: f32 = 1920. / 1080.;
const HEIGHT_TO_WIDTH_RATIO: f32 = 1080. / 1920.;

const PADDING_RIGHT: f32 = 20.;
const PADDING_BOTTOM: f32 = 20.;

pub struct MediaPlayer {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    pub video_stream: Option<Arc<Mutex<video::VideoStream>>>,
    pub texture: Option<Arc<Mutex<texture::Texture>>>,
    pub pipeline: Option<pipeline::Pipeline>,
}

impl Default for MediaPlayer {
    fn default() -> Self {
        Self {
            x: 20.,
            y: 20.,
            width: 1920.,
            height: 1080.,
            video_stream: None,
            texture: None,
            pipeline: None,
        }
    }
}

impl MediaPlayer {
    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        let new_size_width = new_size.width as f32 - self.x - PADDING_RIGHT;
        let new_size_height = new_size.height as f32 - self.y - PADDING_BOTTOM;

        if new_size_width <= 1920. || new_size_height <= 1080. {
            self.height = new_size_width / WIDTH_TO_HEIGHT_RATIO;
            self.width = self.height / HEIGHT_TO_WIDTH_RATIO;

            if new_size_height <= self.height {
                self.width = new_size_height / HEIGHT_TO_WIDTH_RATIO;
                self.height = self.width / WIDTH_TO_HEIGHT_RATIO;
            }
        }
    }

    pub fn uniforms(&self) -> Uniforms {
        Uniforms {
            rect: [self.x, self.y, self.width, self.height],
        }
    }

    pub fn create_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()> {
        let texture = texture::Texture::new(device, queue, &self.uniforms(), None)?;
        self.texture = Some(Arc::new(Mutex::new(texture)));
        Ok(())
    }

    pub fn create_pipeline(&mut self, device: &wgpu::Device, format: wgpu::TextureFormat) {
        let texture = self.texture.as_ref().unwrap().clone();
        let pipeline = pipeline::Pipeline::new(device, format, texture);
        self.pipeline = Some(pipeline);
    }

    pub fn render_video(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window: &winit::window::Window,
    ) -> anyhow::Result<()> {
        let video_stream = self.video_stream.as_ref().unwrap().clone();
        let uniforms = self.uniforms();

        let mut video_stream = video_stream.lock().unwrap();
        let mut scaler = ffmpeg::software::scaling::Context::get(
            video_stream.video_decoder.format(),
            video_stream.video_decoder.width(),
            video_stream.video_decoder.height(),
            ffmpeg::format::Pixel::RGBA,
            video_stream.video_decoder.width(),
            video_stream.video_decoder.height(),
            ffmpeg::software::scaling::Flags::BILINEAR,
        )?;

        let frame_rate = video_stream.frame_rate;

        let wait_duration = std::time::Duration::from_millis((1000. / frame_rate) as u64);

        while let Some((s, p)) = video_stream.get_next_packet() {
            if s.index() == video_stream.video_stream_index {
                video_stream.video_decoder.send_packet(&p)?;

                let mut decoded = ffmpeg::util::frame::Video::empty();

                if video_stream
                    .video_decoder
                    .receive_frame(&mut decoded)
                    .is_ok()
                {
                    let mut rgba_frame = ffmpeg::util::frame::Video::empty();
                    scaler.run(&decoded, &mut rgba_frame)?;

                    let texture =
                        texture::Texture::new(device, queue, &uniforms, Some(rgba_frame))?;
                    self.texture = Some(Arc::new(Mutex::new(texture)));

                    window.request_redraw();
                }
            }
        }

        video_stream.video_decoder.send_eof()?;

        Ok(())
    }
}
