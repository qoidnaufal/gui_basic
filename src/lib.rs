pub mod buffer;
pub mod media;
pub mod texture;
pub mod vertex_buffer;
pub mod video_pipeline;
pub mod window_state;

use media::video;
// use media::video;
use vertex_buffer::Vertex;

use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[repr(C)]
pub struct Uniforms {
    pub rect: [f32; 4],
}

// --- positions are counter-clockwise ordered
// --- tex_coords works like this:
//    (0)------------------(3) --> index
//     | [0, 0]      [1, 0] |
//     |                |   |
//     |                `--------> tex_coords
//     |                    |
//     |                    |
//     |       (0, 0) -----------> center position
//     |                    |
//     |                    |
//     |                    |
//     |                    |
//     | [0, 1]      [1, 1] |
//    (1)------------------(2)

#[rustfmt::skip]
pub const RECT_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.7, 0.7, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.7, -0.7, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [0.7, -0.7, 0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [0.7, 0.7, 0.0], tex_coords: [1.0, 0.0] },
];

#[rustfmt::skip]
pub const RECT_INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0
];

#[rustfmt::skip]
pub const VID_UNIFORMS: Uniforms = Uniforms {
    rect: [0.0, 0.0, 0.7, 0.7]
};

pub struct App<'a> {
    pub window_state: window_state::WindowState<'a>,
    pub video_data: video::VideoStreamData,
}

impl<'a> App<'a> {
    // later on i can change the function's parameter to include event loop
    pub fn new() -> Self {
        Self {
            window_state: window_state::WindowState::new(),
            video_data: video::VideoStreamData::new(),
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
        let bg_color = &[0.824, 0.902, 0.698, 1.0];
        self.window_state.init(bg_color);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => match self.window_state.render(&self.video_data) {
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
                    if let Ok(Some(_)) = self.video_data.open_video() {
                        self.video_data.video_index = 0;
                        self.window_state.window.as_ref().unwrap().request_redraw();
                    }
                }
                KeyCode::ArrowRight => {
                    self.video_data.video_index += 1;
                    if self
                        .video_data
                        .data
                        .lock()
                        .unwrap()
                        .contains_key(&self.video_data.video_index)
                    {
                        self.window_state.window.as_ref().unwrap().request_redraw();
                    } else {
                        log::info!("Reached end...")
                    }
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
