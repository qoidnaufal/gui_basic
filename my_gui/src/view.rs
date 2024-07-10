use wgpu::util::DeviceExt;

use crate::elements::{Button, IntoElement};
use crate::vertex::Vertex;

#[rustfmt::skip]
const RECT_INDICES: &[u16] = &[
    0, 1, 2,
    0, 2, 3
];

#[derive(Clone)]
pub enum View {
    Button(Button),
}

impl View {
    pub fn vertices(&self, size: &winit::dpi::PhysicalSize<u32>) -> [Vertex; 4] {
        match self {
            Self::Button(button) => button.vertices(size),
        }
    }

    pub fn indices(&self, base: u32) -> [u32; 6] {
        match self {
            Self::Button(button) => button.indices(base),
        }
    }

    /* pub fn rgba(&self) -> Vec<u8> {
        match self {
            Self::Button(button) => button.rgba(),
        }
    } */

    /* pub fn bg_layout_entries(&self) -> &[wgpu::BindGroupLayoutEntry; 2] {
        &[
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
        ]
    } */

    pub fn tex_view(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
        match self {
            Self::Button(button) => button.texture_view(device, queue),
        }
    }

    pub fn sampler(&self, device: &wgpu::Device) -> wgpu::Sampler {
        match self {
            Self::Button(button) => button.sampler(device),
        }
    }

    pub fn num_indices(&self) -> u32 {
        match self {
            Self::Button(_) => RECT_INDICES.len() as u32,
        }
    }
}

pub trait IntoView {
    fn into_view(self) -> View;
}

pub fn vertex_buffer(device: &wgpu::Device, vertices: Vec<Vertex>) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(vertices.as_slice()),
        usage: wgpu::BufferUsages::VERTEX,
    })
}

pub fn index_buffer(device: &wgpu::Device, indices: Vec<u32>) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Index Buffer"),
        contents: bytemuck::cast_slice(indices.as_slice()),
        usage: wgpu::BufferUsages::INDEX,
    })
}

pub fn render_pipeline(
    device: &wgpu::Device,
    config: &wgpu::SurfaceConfiguration,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::include_wgsl!("../shaders/shader.wgsl"));

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: "vs_main",
            buffers: &[Vertex::desc()],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: config.format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}
