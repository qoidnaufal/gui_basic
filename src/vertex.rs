use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

// --- positions are counter-clockwise ordered
// --- position & tex_coords works like this:
//
//    (0)---------------------------(3) --> index --- based on
//     | [0.0, 0.0]       [1.0, 0.0] |                       |
//     |                         |   |                       |
//     |                         |   |                       |
//     |                         `--------> tex_coords <-----
//     |                             |
//     |          (0.0, 0.0) -------------> center position
//     |                             |
//     |                             |
//     |                             |
//     |                             |
//     | [0.0, 1.0]       [1.0, 1.0] |
//    (1)---------------------------(2)

#[rustfmt::skip]
const RECT_VERTICES: &[Vertex] = &[
    Vertex { position: [-0.7,  0.7, 0.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.7, -0.7, 0.0], tex_coords: [0.0, 1.0] },
    Vertex { position: [ 0.7, -0.7, 0.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.7,  0.7, 0.0], tex_coords: [1.0, 0.0] },
];

#[rustfmt::skip]
const RECT_INDICES: &[u16] = &[
    0, 1, 2,
    2, 3, 0
];

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub fn create_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    }
}

pub struct VertexBuffer {
    pub vertices: wgpu::Buffer,
    pub index: wgpu::Buffer,
    pub num_indices: u32,
}

impl VertexBuffer {
    pub fn init(device: &wgpu::Device) -> Self {
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(RECT_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(RECT_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let num_indices = RECT_INDICES.len() as u32;

        Self {
            vertices,
            index,
            num_indices,
        }
    }
}
