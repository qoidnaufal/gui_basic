use std::collections::BTreeMap;

use ffmpeg_the_third as ffmpeg;
use wgpu::util::DeviceExt;
use winit::window::Window;

use super::{RECT_INDICES, RECT_VERTICES};

use crate::{texture, vertex_buffer::Vertex};

#[derive(Debug)]
pub struct VideoStreamData {
    pub video_id: usize,
    pub dimensions: (u32, u32),
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct WindowState<'a> {
    pub bg_color: &'a [f64; 4],
    pub window: Option<Window>,
    pub size: Option<winit::dpi::PhysicalSize<u32>>,
    // --- surface
    surface: Option<wgpu::Surface<'a>>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    config: Option<wgpu::SurfaceConfiguration>,
    render_pipeline: Option<wgpu::RenderPipeline>,
    // --- vertex shader
    vertex_buffer: Option<wgpu::Buffer>,
    index_buffer: Option<wgpu::Buffer>,
    num_indices: u32,
    // uniform_buffer: Option<wgpu::Buffer>,
    // --- image
    media_bind_group: Option<wgpu::BindGroup>,
    texture: Option<texture::Texture>,
    // --- video
    pub video_data: BTreeMap<usize, VideoStreamData>,
    pub video_index: usize,
}

impl<'a> WindowState<'a> {
    pub fn new() -> Self {
        Self {
            bg_color: &[0.0, 0.0, 0.0, 1.0],
            surface: None,
            device: None,
            queue: None,
            config: None,
            size: None,
            window: None,
            render_pipeline: None,
            vertex_buffer: None,
            index_buffer: None,
            num_indices: 0,
            // uniform_buffer: None,
            media_bind_group: None,
            texture: None,
            video_data: BTreeMap::new(),
            video_index: 0,
        }
    }

    pub fn init(&mut self) {
        let bg_color = &[0.824, 0.902, 0.698, 1.0];
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

        // let video_pipeline = video_pipeline::VideoPipeline::new(&device, config.format);
        // self.video_pipeline = Some(video_pipeline);

        let texture =
            texture::Texture::from_bytes(&device, &queue, &self.video_data, self.video_index)
                .unwrap();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    // ---  new here, correspond with new shader
                    // wgpu::BindGroupLayoutEntry {
                    //     binding: 2,
                    //     visibility: wgpu::ShaderStages::VERTEX,
                    //     ty: wgpu::BindingType::Buffer {
                    //         ty: wgpu::BufferBindingType::Uniform,
                    //         has_dynamic_offset: false,
                    //         min_binding_size: None,
                    //     },
                    //     count: None,
                    // },
                ],
            });

        let media_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
                // wgpu::BindGroupEntry {
                //     binding: 2,
                //     resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                //         buffer: &uniform_buffer,
                //         offset: 0,
                //         size: None,
                //     }),
                // },
            ],
            label: Some("image Bind Group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("../shader/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[Vertex::create_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None, // None
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.bg_color = bg_color;
        self.device = Some(device);
        self.queue = Some(queue);
        self.config = Some(config);
        self.size = Some(size);
        self.surface =
            Some(unsafe { std::mem::transmute::<wgpu::Surface<'_>, wgpu::Surface<'a>>(surface) });
        self.render_pipeline = Some(render_pipeline);

        self.vertex_buffer = Some(vertex_buffer);
        self.index_buffer = Some(index_buffer);
        self.num_indices = RECT_INDICES.len() as u32;
        self.media_bind_group = Some(media_bind_group);
        self.texture = Some(texture);

        // self.uniform_buffer = Some(uniform_buffer);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size.replace(new_size);
            self.config.as_mut().map(|c| {
                c.width = new_size.width;
                c.height = new_size.height;
            });
            self.surface.as_mut().map(|s| {
                s.configure(
                    &self.device.as_ref().unwrap(),
                    &self.config.as_ref().unwrap(),
                )
            });
            self.window.as_ref().map(|w| w.request_redraw());
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
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
                            r: self.bg_color[0], // 0.1
                            g: self.bg_color[1], // 0.2
                            b: self.bg_color[2], // 0.3
                            a: self.bg_color[3],
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(self.render_pipeline.as_ref().unwrap());
            render_pass.set_bind_group(0, self.media_bind_group.as_ref().unwrap(), &[]);

            render_pass.set_vertex_buffer(0, self.vertex_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(
                self.index_buffer.as_ref().unwrap().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue
            .as_ref()
            .unwrap()
            .submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }

    pub fn open_video(&mut self) -> Result<Option<()>, ffmpeg::Error> {
        let maybe_path = rfd::FileDialog::new().pick_file();

        if let Some(path) = maybe_path {
            ffmpeg::init().unwrap();
            log::info!("{:?}", path);

            // --- read the file from the path
            let mut input = ffmpeg::format::input(&path)?;
            let video_stream = input
                .streams()
                .best(ffmpeg::util::media::Type::Video)
                .ok_or(ffmpeg::Error::StreamNotFound)?;

            let video_stream_index = video_stream.index();

            let decoder_context =
                ffmpeg::codec::Context::from_parameters(video_stream.parameters())?;
            // --- ffmpeg::codec::decoder::video::Video
            let mut packet_decoder = decoder_context.decoder().video()?;

            let mut scaler = ffmpeg::software::scaling::Context::get(
                packet_decoder.format(),
                packet_decoder.width(),
                packet_decoder.height(),
                ffmpeg::format::Pixel::RGBA,
                packet_decoder.width(),
                packet_decoder.height(),
                ffmpeg::software::scaling::Flags::BILINEAR,
            )?;

            let mut frame_idx = 0usize;

            let mut receive_decoded_frame = |p_dec: &mut ffmpeg::codec::decoder::video::Video| {
                let mut decoded = ffmpeg::util::frame::Video::empty();
                if p_dec.receive_frame(&mut decoded).is_ok() {
                    let mut rgba_frame = ffmpeg::util::frame::Video::empty();
                    scaler.run(&decoded, &mut rgba_frame)?;

                    self.video_data.insert(
                        frame_idx,
                        VideoStreamData {
                            video_id: frame_idx,
                            dimensions: (rgba_frame.width(), rgba_frame.height()),
                            data: rgba_frame.data(0).to_vec(),
                        },
                    );

                    frame_idx += 1;
                }

                Ok::<(), ffmpeg::Error>(())
            };

            while let Some(Ok((stream, packet))) = input.packets().next() {
                if stream.index() == video_stream_index {
                    packet_decoder.send_packet(&packet)?;
                    receive_decoded_frame(&mut packet_decoder)?;
                }
            }

            packet_decoder.send_eof()?;
            receive_decoded_frame(&mut packet_decoder)?;

            Ok(Some(()))
        } else {
            Ok(None)
        }
    }
}
