use super::IntoElement;
use crate::vertex::Vertex;
use crate::view::{IntoView, View};

#[derive(Clone)]
pub struct Button {
    pub top: u32,
    pub left: u32,
    pub bottom: u32,
    pub right: u32,
    pub color: [f32; 4],
}

impl Default for Button {
    fn default() -> Self {
        let position = [10, 10, 50, 120];
        let color = [0.235, 0.639, 0.282, 1.];

        Self {
            top: position[0],
            left: position[1],
            bottom: position[2],
            right: position[3],
            color,
        }
    }
}

impl Button {
    pub fn color(&self) -> [f32; 4] {
        self.color
    }

    pub fn set_color(&mut self, color: [f32; 4]) {
        self.color = color;
    }

    pub fn position(&self) -> [u32; 4] {
        [self.top, self.left, self.bottom, self.right]
    }

    pub fn set_position(&mut self, position: [u32; 4]) {
        self.top = position[0];
        self.left = position[1];
        self.bottom = position[2];
        self.right = position[3];
    }
}

impl IntoElement for Button {
    fn color(&self) -> [f32; 4] {
        self.color()
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

impl IntoView for Button {
    fn into_view(self) -> View {
        View::Button(self)
    }
}

impl IntoView for &Button {
    fn into_view(self) -> View {
        View::Button(self.to_owned())
    }
}
