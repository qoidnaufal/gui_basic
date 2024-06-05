use winit::window::Window;

use crate::media::media_player;

pub struct WindowState<'a> {
    pub bg_color: &'a [f64; 4],
    pub window: Option<Window>,
    pub size: Option<winit::dpi::PhysicalSize<u32>>,
    // --- surface
    pub surface: Option<wgpu::Surface<'a>>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub config: Option<wgpu::SurfaceConfiguration>,
}

impl Default for WindowState<'_> {
    fn default() -> Self {
        Self {
            bg_color: &[0.0, 0.0, 0.0, 1.0],
            surface: None,
            device: None,
            queue: None,
            config: None,
            size: None,
            window: None,
        }
    }
}

impl<'a> WindowState<'a> {
    pub fn init(&mut self, bg_color: &'a [f64; 4]) {
        let window = self.window.as_ref().unwrap();

        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();

        let (device, queue, config) = futures::executor::block_on(async {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::default(),
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
                .unwrap();

            let (device, queue) = adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: None,
                        required_features: wgpu::Features::empty(),
                        required_limits: wgpu::Limits::default(),
                    },
                    None,
                )
                .await
                .unwrap();

            let surface_capabilities = surface.get_capabilities(&adapter);

            // surface_format: wgpu::TextureFormat
            let surface_format = surface_capabilities
                .formats
                .iter()
                .find(|sf| sf.is_srgb())
                .copied()
                .unwrap_or(surface_capabilities.formats[0]);

            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width,
                height: size.height,
                present_mode: surface_capabilities.present_modes[0],
                alpha_mode: surface_capabilities.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            (device, queue, config)
        });

        surface.configure(&device, &config);

        // ------------------------------------------

        self.bg_color = bg_color;
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.size = Some(size);
        self.surface =
            Some(unsafe { std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'a>>(surface) });
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size.replace(new_size);

            if let Some(c) = self.config.as_mut() {
                c.width = new_size.width;
                c.height = new_size.height;
            };
            if let Some(s) = self.surface.as_mut() {
                s.configure(
                    &self.device.as_ref().unwrap(),
                    &self.config.as_ref().unwrap(),
                )
            };
        }
    }

    // --- this function render the whole window
    pub fn render_window(
        &mut self,
        media_player: &mut media_player::MediaPlayer,
    ) -> Result<(), wgpu::SurfaceError> {
        if media_player.texture.as_ref().is_none() {
            media_player
                .create_texture(self.device.as_ref().unwrap(), self.queue.as_ref().unwrap())
                .unwrap();
        }

        media_player.create_pipeline(
            self.device.as_ref().unwrap(),
            self.config.as_ref().unwrap().format,
        );

        let pipeline = media_player.pipeline.as_ref().unwrap();

        // output: SurfaceTexture
        let output = self.surface.as_ref().unwrap().get_current_texture()?;

        // view: TextureView
        let view = output.texture.create_view(&Default::default());

        let mut encoder =
            self.device
                .as_ref()
                .unwrap()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.bg_color[0],
                            g: self.bg_color[1],
                            b: self.bg_color[2],
                            a: self.bg_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // ---- the loop needs to happen here _.
            //                                     |
            //                                     v
            render_pass.set_pipeline(&pipeline.render_pipeline);
            render_pass.set_bind_group(0, &pipeline.bind_group, &[]);
            render_pass.set_viewport(
                media_player.uniforms().rect[0],
                media_player.uniforms().rect[1],
                media_player.uniforms().rect[2],
                media_player.uniforms().rect[3],
                0.0,
                1.0,
            );
            render_pass.draw(0..4, 0..1);
        }

        self.queue
            .as_ref()
            .unwrap()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
