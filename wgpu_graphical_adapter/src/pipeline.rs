use crate::model::UnloadedModel;

pub trait WgpuGraphicalAdapterPipelineFactory {
    fn create(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Box<dyn WgpuGraphicalAdapterPipeline>;
}

pub trait WgpuGraphicalAdapterPipeline {
    fn load_model_sync(&mut self,
                       model: UnloadedModel,
                       image_loader_gateway: &dyn common::gateways::ImageLoaderGateway,
                       device: &wgpu::Device,
                       queue: &wgpu::Queue,
    ) -> anyhow::Result<()>;
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}