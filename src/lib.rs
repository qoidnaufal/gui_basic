pub mod video;
pub mod video_player;
pub mod window_state;

// use ffmpeg_the_third as ffmpeg;
use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[derive(Debug)]
pub struct App<'a> {
    pub window_state: window_state::WindowState<'a>,
}

impl<'a> App<'a> {
    // later on i can change the function's parameter to include event loop
    pub fn new() -> Self {
        Self {
            window_state: window_state::WindowState::new(),
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_maximized(false)
                    .with_title("My Basic GUI"),
            )
            .unwrap();

        self.window_state.window = Some(window);
        self.window_state.init();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => match self.window_state.render() {
                Ok(_) => (),
                Err(wgpu::SurfaceError::Lost) => {
                    self.window_state.resize(self.window_state.size.unwrap())
                }
                Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                Err(err) => log::error!("{:?}", err),
            },
            WindowEvent::Resized(physical_size) => {
                self.window_state.resize(physical_size);
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let new_width =
                    (self.window_state.size.unwrap().width as f64 * scale_factor) as u32;
                let new_height =
                    (self.window_state.size.unwrap().height as f64 * scale_factor) as u32;
                let new_size = winit::dpi::PhysicalSize::new(new_width, new_height);
                self.window_state.resize(new_size);
            }
            WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        state: event::ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key),
                        ..
                    },
                ..
            } => {
                match key {
                    KeyCode::KeyO => {
                        let _video_file = video::VideoFile::open_file();
                        // video_file.decode_file()?;
                    }
                    _ => {
                        log::info!("pressed key: {:?}", key)
                    }
                }
            }
            _ => (),
            // n => log::info!("{:?}", n),
        }
    }
}
