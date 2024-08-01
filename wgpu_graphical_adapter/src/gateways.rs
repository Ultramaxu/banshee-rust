use crate::instance::Instance;
use crate::model::Model;

pub trait WgpuModelLoaderGateway {
    fn load_model_sync(
        &self,
        file_name: &str,
        instances: Vec<Instance>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
        bind_group_builder: Box<dyn Fn(
            &wgpu::Device,
            &wgpu::TextureView,
            &wgpu::Sampler,
            &wgpu::BindGroupLayout
        ) -> wgpu::BindGroup>,
    ) -> anyhow::Result<Model>;
}