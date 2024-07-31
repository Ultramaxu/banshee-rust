use wgpu::util::DeviceExt;
use crate::vertex::Vertex;
use crate::texture::Texture;

pub struct UnloadedModel {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture_id: String,
}

impl UnloadedModel {
    pub fn load<F>(
        &self,
        image_loader_gateway: &dyn common::gateways::ImageLoaderGateway,
        bind_group_builder: F,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<Model> where F: FnOnce(&wgpu::TextureView, &wgpu::Sampler) -> wgpu::BindGroup {
        let texture = Texture::load(
            image_loader_gateway,
            bind_group_builder,
            device,
            queue,
            &self.texture_id
        )?;

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&self.indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        
        Ok(Model {
            vertex_buffer,
            index_buffer,
            texture,
            num_indices: self.indices.len() as u32,
        })
    }
}

pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub texture: Texture,
    pub num_indices: u32,
}

impl Model {
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, &self.texture.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}