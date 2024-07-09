use crate::vertex::Vertex;
use super::IntoView;

pub struct Button {
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
    pub color: [f32; 4],
}

impl IntoView for Button {
    fn new(position: [u32; 4], color: [f32; 4]) -> Self {
        Self {
            top: position[0],
            left: position[1],
            bottom: position[2],
            right: position[3],
            color,
        }
    }

    fn color(&self) -> [f32; 4] {
        self.color
    }

    #[rustfmt::skip]
    fn vertices(&self, size: &winit::dpi::PhysicalSize<u32>) -> [Vertex; 4] {
        let top    = 1. - (self.top    as f32 / (size.height as f32 / 2.));
        let left   = (self.left as f32 / (size.width as f32 / 2.)) - 1.;
        let bottom = 1. - (self.bottom as f32 / (size.height as f32 / 2.));
        let right  = (self.right as f32 / (size.width as f32 / 2.)) - 1.;
        
        [
            Vertex { position: [left,  top,    0.], tex_coords: [0., 0.] },
            Vertex { position: [left,  bottom, 0.], tex_coords: [0., 1.] },
            Vertex { position: [right, bottom, 0.], tex_coords: [1., 1.] },
            Vertex { position: [right, top,    0.], tex_coords: [1., 0.] },
        ]
    }
}
