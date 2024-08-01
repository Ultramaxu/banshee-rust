use common::gateways::ImageLoaderGatewayResult;

pub struct Texture {
    pub diffuse_bind_group: wgpu::BindGroup,
}

impl Texture {
    pub fn load<F>(
        raw_texture_data: ImageLoaderGatewayResult,
        bind_group_builder: F,
        device: &wgpu::Device,
        queue: &wgpu::Queue
    ) -> anyhow::Result<Texture> where F: FnOnce(&wgpu::TextureView, &wgpu::Sampler) -> wgpu::BindGroup {

        let texture_size = wgpu::Extent3d {
            width: raw_texture_data.dimensions.0,
            height: raw_texture_data.dimensions.1,
            depth_or_array_layers: 1,
        };

        let diffuse_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                // All textures are stored as 3D, we represent our 2D texture
                // by setting depth to 1.
                size: texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // Most images are stored using sRGB, so we need to reflect that here.
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                // TEXTURE_BINDING tells wgpu that we want to use this texture in shaders
                // COPY_DST means that we want to copy data to this texture
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None,
                // This is the same as with the SurfaceConfig. It
                // specifies what texture formats can be used to
                // create TextureViews for this texture. The base
                // texture format (Rgba8UnormSrgb in this case) is
                // always supported. Note that using a different
                // texture format is not supported on the WebGL2
                // backend.
                view_formats: &[],
            }
        );

        queue.write_texture(
            // Tells wgpu where to copy the pixel data
            wgpu::ImageCopyTexture {
                texture: &diffuse_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            // The actual pixel data
            &raw_texture_data.data,
            // The layout of the texture
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * raw_texture_data.dimensions.0),
                rows_per_image: Some(raw_texture_data.dimensions.1),
            },
            texture_size,
        );

        // We don't need to configure the texture view much, so let's
        // let wgpu define it.
        let diffuse_view = diffuse_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // FIXME with proper tests and read about mipmaps
        let diffuse_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
            // Other parameters are at https://docs.rs/wgpu/latest/wgpu/struct.SamplerDescriptor.html
        });

        let diffuse_bind_group = bind_group_builder(&diffuse_view, &diffuse_sampler);

        Ok(Texture {
            diffuse_bind_group,
        })
    }
}