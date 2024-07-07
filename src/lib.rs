use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

mod pipeline;
mod texture;
mod uniforms;
mod vertex;
mod window_state;

pub struct App<'a> {
    pub bg_color: &'a [f64; 4],
    pub window_state: window_state::WindowState<'a>,
}

impl Default for App<'_> {
    fn default() -> Self {
        Self {
            bg_color: &[1., 1., 1., 1.],
            window_state: window_state::WindowState::default(),
        }
    }
}

impl<'a> App<'a> {
    pub fn set_bg_color(&mut self, input: &'a [f64; 4]) {
        self.bg_color = input;
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

        self.window_state.init(self.bg_color);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => match self.window_state.render_window() {
                Ok(_) => (),
                Err(wgpu::SurfaceError::Lost) => {
                    self.window_state.resize(self.window_state.size.unwrap())
                }
                Err(wgpu::SurfaceError::OutOfMemory) => {
                    event_loop.exit();
                }
                Err(err) => log::error!("Render Error: {:?}", err),
            },
            WindowEvent::Resized(physical_size) => {
                self.window_state.resize(physical_size);
                self.window_state.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                let new_width =
                    (self.window_state.size.unwrap().width as f64 * scale_factor) as u32;
                let new_height =
                    (self.window_state.size.unwrap().height as f64 * scale_factor) as u32;

                let new_size = winit::dpi::PhysicalSize::new(new_width, new_height);

                self.window_state.resize(new_size);
                self.window_state.window.as_ref().unwrap().request_redraw();
            }
            // ------------------------------------------------
            _ => (),
            // n => log::info!("{:?}", n),
        }
    }
}
