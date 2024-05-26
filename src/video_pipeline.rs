use std::collections::BTreeMap;

#[repr(C)]
pub struct Uniforms {
    rect: [f32; 4],
}

pub struct VideoPipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub sampler: wgpu::Sampler,
    pub texture: BTreeMap<usize, (wgpu::Texture, wgpu::Buffer, wgpu::BindGroup)>,
}

impl VideoPipeline {
    // --- format should use config.format?
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let video_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Video Player Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader/video_shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Video Player Bind Group Layout"),
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
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        // --- bind_group is not defined here, but on the fn prepare()

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Video Player Layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Video Player Pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &video_shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[], // there's no VertexBufferLayout here?
            },
            fragment: Some(wgpu::FragmentState {
                module: &video_shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None, // there's no BlendState here
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Video Player Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        });

        Self {
            pipeline,
            bind_group_layout,
            sampler,
            texture: BTreeMap::new(),
        }
    }

    pub fn write_texture(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        video_id: usize,
        (width, height): (u32, u32),
        frame: &[u8],
    ) {
        if !self.texture.contains_key(&video_id) {
            // ---  on the tutorial, texture is obtained via `self.surface.as_ref().unwrap().get_current_texture()?;`
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Video Player Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });

            // --- on the tutorial, view is .create_view(Default::default())
            let view = texture.create_view(&wgpu::TextureViewDescriptor {
                label: Some("Video Player Texture View"),
                format: None,
                dimension: None,
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: None,
                base_array_layer: 0,
                array_layer_count: None,
            });

            let buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Video Player Uniform Buffer"),
                size: std::mem::size_of::<Uniforms>() as _,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                mapped_at_creation: false,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Video Player Bind Group"),
                layout: &self.bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                            buffer: &buffer,
                            offset: 0,
                            size: None,
                        }),
                    },
                ],
            });

            self.texture.insert(video_id, (texture, buffer, bind_group));
        }

        let (texture, _, _) = self.texture.get(&video_id).unwrap();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            frame,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
    }
}
