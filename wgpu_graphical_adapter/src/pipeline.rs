use crate::camera::PerspectiveCamera;
use crate::instance::Instance;

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
                       id: &str,
                       filename: &str,
                       instances: Vec<Instance>,
                       device: &wgpu::Device,
                       queue: &wgpu::Queue,
    ) -> anyhow::Result<()>;
    fn update_camera(&mut self, camera: &PerspectiveCamera, queue: &wgpu::Queue);
    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>);
    fn update_model_instances(&mut self, model_id: &str, instances: Vec<Instance>, device: &wgpu::Device) -> anyhow::Result<()>;
}