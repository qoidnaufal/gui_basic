use crate::{
    view::{index_buffer, render_pipeline, vertex_buffer},
    View,
};

use winit::window::Window;

pub struct WindowContext<'a> {
    pub bg_color: &'a [f64; 4],
    pub window: Option<Window>,
    pub size: Option<winit::dpi::PhysicalSize<u32>>,
    // --- surface
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
}

impl Default for WindowContext<'_> {
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

impl<'a> WindowContext<'a> {
    pub fn set_window(&mut self, window: Window) {
        self.window.replace(window);
    }

    pub fn set_bg_color(&mut self, bg_color: &'a [f64; 4]) {
        self.bg_color = bg_color;
    }

    pub fn init(&mut self) {
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
                .find(|tx_fmt| tx_fmt.is_srgb())
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
                s.configure(self.device.as_ref().unwrap(), self.config.as_ref().unwrap())
            };
        }
    }

    // --- this function render the whole window
    pub fn render(&mut self, components: &Vec<View>) -> Result<(), wgpu::SurfaceError> {
        let mut vertex_buf = Vec::new();
        let mut index_buf = Vec::new();
        let mut texture_view_array = Vec::new();
        let mut sampler_array = Vec::new();
        // let mut rgba = Vec::new();

        let mut num_vertices = 0;
        let mut num_indices = 0;

        for component in components {
            let vertices = component.vertices(self.size.as_ref().unwrap());
            vertex_buf.extend_from_slice(vertices.as_slice());

            let indices = component.indices(num_vertices);
            index_buf.extend_from_slice(indices.as_slice());

            num_vertices += vertices.len() as u32;
            num_indices += component.num_indices();

            let tv =
                component.tex_view(self.device.as_ref().unwrap(), self.queue.as_ref().unwrap());
            texture_view_array.push(tv);

            let s = component.sampler(self.device.as_ref().unwrap());
            sampler_array.push(s);

            // let color = component.rgba();
            // rgba.extend_from_slice(color.as_slice());
        }

        let bind_group_layout = self.device.as_ref().unwrap().create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        );

        let bind_group =
            self.device
                .as_ref()
                .unwrap()
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Bind Group"),
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureViewArray(
                                texture_view_array.iter().collect::<Vec<_>>().as_slice(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::SamplerArray(
                                sampler_array.iter().collect::<Vec<_>>().as_slice(),
                            ),
                        },
                    ],
                });

        let pipeline = render_pipeline(
            self.device.as_ref().unwrap(),
            self.config.as_ref().unwrap(),
            &bind_group_layout,
        );

        // vertex buffer
        let vertex_buffer = vertex_buffer(self.device.as_ref().unwrap(), vertex_buf);
        let index_buffer = index_buffer(self.device.as_ref().unwrap(), index_buf);

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

            render_pass.set_pipeline(&pipeline); // pipeline -> button
            render_pass.set_bind_group(0, &bind_group, &[]); // pipeline -> button
            render_pass.set_vertex_buffer(0, vertex_buffer.slice(..)); // buffer -> button
            render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16); // buffer -> button
            render_pass.draw_indexed(0..num_indices, 0, 0..1); // buffer -> button
        }

        self.queue
            .as_ref()
            .unwrap()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}
