use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::video_player::VideoPlayer;

#[derive(Default)]
pub struct App<'a> {
    pub surface: Option<wgpu::Surface<'a>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
    pub size: Option<winit::dpi::PhysicalSize<u32>>,
    pub window: Option<Window>,
}

impl<'a> App<'a> {
    pub fn init(&mut self, window: Window) {
        let size = window.inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap())
                .unwrap()
        };

        self.window = Some(window);
        self.surface = Some(surface);
        self.size = Some(size);
    }

    pub async fn set(&mut self) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&Default::default())
            .await
            .expect("Unable to request adapter");
        let (device, queue) = adapter
            .request_device(&Default::default(), None)
            .await
            .unwrap();
        let surface_capabilities = &self.surface.as_ref().unwrap().get_capabilities(&adapter);
        let surface_format = surface_capabilities
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_capabilities.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: self
                .size
                .unwrap_or_else(|| winit::dpi::PhysicalSize::new(50, 50))
                .width,
            height: self
                .size
                .unwrap_or_else(|| winit::dpi::PhysicalSize::new(50, 50))
                .height,
            present_mode: surface_capabilities.present_modes[0],
            alpha_mode: surface_capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
    }

    pub fn _window(&self) -> &Window {
        todo!()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = Some(new_size);
            self.config.as_mut().map(|conf| conf.width = new_size.width);
            self.config
                .as_mut()
                .map(|conf| conf.height = new_size.height);
            self.surface.as_mut().map(|surf| {
                surf.configure(
                    &self.device.as_ref().unwrap(),
                    &self.config.as_ref().unwrap(),
                )
            });
        }
    }

    pub fn _input(&mut self, _event: &WindowEvent) -> bool {
        todo!()
    }

    pub fn update(&mut self) {}

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.as_ref().unwrap().get_current_texture()?;
        let view = output.texture.create_view(&Default::default());
        let mut encoder =
            self.device
                .as_ref()
                .unwrap()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
        }
        let command_buffer = encoder.finish();
        self.queue
            .as_ref()
            .unwrap()
            .submit(std::iter::once(command_buffer));
        output.present();
        Ok(())
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let config = Window::default_attributes()
            .with_maximized(false)
            .with_title("My Basic GUI");

        let window = event_loop.create_window(config).unwrap();

        self.init(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();

                // --- idk why i can't put this on `resumed` function
                futures::executor::block_on(async move {
                    self.set().await;
                    self.update();
                    match self.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => self.resize(self.size.unwrap()),
                        Err(wgpu::SurfaceError::OutOfMemory) => event_loop.exit(),
                        Err(err) => eprintln!("{:?}", err),
                    }
                });
            }
            // WindowEvent::Resized(physical_size) => {
            //     self.resize(physical_size);
            // }
            // WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
            //     let width = self.size.unwrap().width as f64 * scale_factor;
            //     let height = self.size.unwrap().height as f64 * scale_factor;
            //     let new_size = winit::dpi::PhysicalSize::new(width as u32, height as u32);
            //     self.resize(new_size);
            // }
            WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        state: event::ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyO),
                        ..
                    },
                ..
            } => {
                // another option to spawn std::thread
                // and use futures::executor to block on and run the async code

                // std::thread::Builder::new()
                //     .name("open file".into())
                //     .spawn(move || {
                //         futures::executor::block_on(async move {});
                //     })
                //     .unwrap()
                //     .join()
                //     .unwrap();

                tokio::spawn(async move {
                    let video_player = VideoPlayer::open_file().await;
                    video_player.decode_file();
                });
            }
            _ => (), // n => println!("{:?}", n),
        }
    }
}
