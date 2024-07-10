mod button;

pub use button::Button;

use crate::vertex::Vertex;

pub fn button() -> Button {
    Button::default()
}

pub trait IntoElement {
    fn color(&self) -> [f32; 4];

    fn vertices(&self, size: &winit::dpi::PhysicalSize<u32>) -> [Vertex; 4];

    #[rustfmt::skip]
    fn indices(&self, base: u32) -> [u32; 6] {
        [
            base, 1 + base, 2 + base,
            base, 2 + base, 3 + base
        ]
    }

    fn rgba(&self) -> Vec<u8> {
        self.color()
            .iter()
            .map(|i| (*i * 255.) as u8)
            .collect::<Vec<_>>()
    }

    fn texture_view(&self, device: &wgpu::Device, queue: &wgpu::Queue) -> wgpu::TextureView {
        let dimensions = (1u32, 1u32); // unicolored shape

        let rgba = self.rgba();

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

        // something sus is happening here
        // my guess is, this thing needs to be called only once
        // but then, how do i define the multiple texture?

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

        texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    fn sampler(&self, device: &wgpu::Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        })
    }
}
