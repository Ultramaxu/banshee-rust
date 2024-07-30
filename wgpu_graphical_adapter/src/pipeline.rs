pub trait WgpuGraphicalAdapterPipelineFactory {
    fn create(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Box<dyn WgpuGraphicalAdapterPipeline>;
}

pub trait WgpuGraphicalAdapterPipeline {
    fn render(&self, render_pass: &mut wgpu::RenderPass);
}