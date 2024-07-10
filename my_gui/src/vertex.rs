use bytemuck::{Pod, Zeroable};

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

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

impl Vertex {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
}
