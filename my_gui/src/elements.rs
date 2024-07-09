use crate::vertex::Vertex;

mod button;

use button::Button;

pub fn button(position: [u32; 4], color: [f32; 4]) -> Button {
    Button::new(position, color)
}

pub trait IntoView {
    fn new(position: [u32; 4], color: [f32; 4]) -> Self;

    fn color(&self) -> [f32; 4];

    fn vertices(&self, size: &winit::dpi::PhysicalSize<u32>) -> [Vertex; 4];

    fn texture(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::Texture {
        let dimensions = (1u32, 1u32);
        let rgba = self
            .color()
            .iter()
            .map(|i| (*i * 255.) as u8)
            .collect::<Vec<_>>();

        let size = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // ----- is this the correct place to do write_texture?

        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        texture
    }
}
