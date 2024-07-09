use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

mod elements;
mod pipeline;
mod texture;
mod vertex;
mod window;

pub use elements::{button, IntoView};

pub struct App<'a> {
    bg_color: &'a [f64; 4],
    window: window::WindowContext<'a>,
    title: &'a str,
    window_size: winit::dpi::PhysicalSize<u32>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            bg_color: &[1., 1., 1., 1.],
            window: window::WindowContext::default(),
            title: "My Basic GUI",
            window_size: winit::dpi::PhysicalSize::new(800, 600),
        }
    }
}

impl<'a> App<'a> {
    pub fn set_bg_color(&mut self, input: &'a [f64; 4]) {
        self.bg_color = input;
    }

    pub fn set_title(&mut self, title: &'a str) {
        self.title = title;
    }

    pub fn set_window_size(&mut self, width: u32, height: u32) {
        self.window_size = winit::dpi::PhysicalSize::new(width, height);
    }

    pub fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        env_logger::init();

        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Wait);

        event_loop.run_app(self).map_err(|err| err.into())
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop
            .create_window(
                Window::default_attributes()
                    .with_maximized(false)
                    .with_inner_size(self.window_size)
                    .with_title(self.title),
            )
            .unwrap();

        self.window.set_window(window);
        self.window.set_bg_color(self.bg_color);
        self.window.init();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => match self.window.render_window() {
                Ok(_) => (),
                Err(wgpu::SurfaceError::Lost) => self.window.resize(self.window.size.unwrap()),
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    event_loop.exit();
                }
                Err(err) => log::error!("Render Error: {:?}", err),
            },
            WindowEvent::Resized(physical_size) => {
                self.window.resize(physical_size);
                self.window.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let new_width = (self.window.size.unwrap().width as f64 * scale_factor) as u32;
                let new_height = (self.window.size.unwrap().height as f64 * scale_factor) as u32;

                let new_size = winit::dpi::PhysicalSize::new(new_width, new_height);

                self.window.resize(new_size);
                self.window.window.as_ref().unwrap().request_redraw();
            }
            // ------------------------------------------------
            _ => (),
        }
    }
}
