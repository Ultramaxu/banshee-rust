use wgpu::util::DeviceExt;
use common::gateways::ImageLoaderGatewayResult;
use crate::instance::Instance;
use crate::vertex::Vertex;
use crate::texture::Texture;

pub struct Model {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    instances: Vec<Instance>,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    instance_buffer: wgpu::Buffer,
    texture: Texture,
}

impl Model {
    pub fn load<F>(
        vertices: Vec<Vertex>,
        indices: Vec<u16>,
        instances: Vec<Instance>,
        raw_texture_data: ImageLoaderGatewayResult,
        bind_group_builder: F,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<Model> where F: FnOnce(&wgpu::TextureView, &wgpu::Sampler) -> wgpu::BindGroup {
        let texture = Texture::load(
            raw_texture_data,
            bind_group_builder,
            device,
            queue
        )?;

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
        let instance_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let num_indices = indices.len() as u32;

        Ok(Model {
            vertices,
            indices,
            instances,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            texture,
            num_indices,
        })
    }
    
    pub fn render(&self, render_pass: &mut wgpu::RenderPass) {
        render_pass.set_bind_group(1, &self.texture.diffuse_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..self.instances.len() as _);
    }
}