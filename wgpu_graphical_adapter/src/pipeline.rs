use common::gateways::ImageLoaderGatewayResult;
use crate::camera::PerspectiveCamera;
use crate::instance::Instance;
use crate::vertex::Vertex;

pub trait WgpuGraphicalAdapterPipelineFactory {
    fn create(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera: &PerspectiveCamera,
    ) -> Box<dyn WgpuGraphicalAdapterPipeline>;
}

pub trait WgpuGraphicalAdapterPipeline {
    fn load_model_sync(&mut self,
                       vertices: Vec<Vertex>,
                       indices: Vec<u16>,
                       instances: Vec<Instance>,
                       raw_texture_data: ImageLoaderGatewayResult,
                       device: &wgpu::Device,
                       queue: &wgpu::Queue,
    ) -> anyhow::Result<()>;
    fn update_camera(&mut self, camera: &PerspectiveCamera, queue: &wgpu::Queue);
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}