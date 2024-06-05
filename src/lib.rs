use std::sync::{Arc, Mutex};
use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

pub mod media;
pub mod texture;
pub mod uniforms;
pub mod vertex;
pub mod window_state;

use media::{media_player, video};

pub struct App<'a> {
    pub window_state: window_state::WindowState<'a>,
    pub media_player: media_player::MediaPlayer,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            window_state: window_state::WindowState::default(),
            media_player: media_player::MediaPlayer::default(),
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_maximized(false)
                    .with_inner_size(winit::dpi::PhysicalSize::new(3000, 1700))
                    .with_min_inner_size(winit::dpi::PhysicalSize::new(888, 500))
                    .with_title("My Basic GUI"),
            )
            .unwrap();

        log::info!("{:?}", window.inner_size());

        self.window_state.window = Some(window);
        let bg_color = &[0.824, 0.902, 0.698, 1.0];
        self.window_state.init(bg_color);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                match self.window_state.render_window(&mut self.media_player) {
                    Ok(_) => (),
                    Err(wgpu::SurfaceError::Lost) => {
                        self.window_state.resize(self.window_state.size.unwrap())
                    }
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        event_loop.exit();
                    }
                    Err(err) => log::error!("Render Error: {:?}", err),
                }
            }
            WindowEvent::Resized(physical_size) => {
                self.window_state.resize(physical_size);
                self.media_player.resize(physical_size);
                self.window_state.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let new_width =
                    (self.window_state.size.unwrap().width as f64 * scale_factor) as u32;
                let new_height =
                    (self.window_state.size.unwrap().height as f64 * scale_factor) as u32;

                let new_size = winit::dpi::PhysicalSize::new(new_width, new_height);

                self.window_state.resize(new_size);
                self.media_player.resize(new_size);
                self.window_state.window.as_ref().unwrap().request_redraw();
            }
            // ------------------------------------------------
            WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        state: event::ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => match key {
                KeyCode::KeyO => {
                    if let Ok(Some(video_stream)) = video::VideoStream::open_video() {
                        self.media_player.video_stream = Some(Arc::new(Mutex::new(video_stream)));
                        let device = self.window_state.device.as_ref().unwrap();
                        let queue = self.window_state.queue.as_ref().unwrap();
                        let window = self.window_state.window.as_ref().unwrap();

                        self.media_player
                            .render_video(device, queue, window)
                            .unwrap();
                    }
                }
                KeyCode::KeyR => {
                    self.media_player
                        .resize(winit::dpi::PhysicalSize::new(1000, 1080));
                    self.window_state.window.as_ref().unwrap().request_redraw();
                }
                _ => {
                    log::info!("pressed key: {:?}", key)
                }
            },
            _ => (),
            // n => log::info!("{:?}", n),
        }
    }
}
