pub trait WgpuGraphicalAdapterPipelineFactory {
    fn create(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Box<dyn WgpuGraphicalAdapterPipeline>;
}

pub trait WgpuGraphicalAdapterPipeline {
    fn get_inner(&self) -> &wgpu::RenderPipeline;
}

pub struct DefaultWgpuGraphicalAdapterPipelineFactory<'a> {
    shader_code: &'a str,
}

impl <'a> DefaultWgpuGraphicalAdapterPipelineFactory<'a> {
    pub fn new(shader_code: &str) -> DefaultWgpuGraphicalAdapterPipelineFactory {
        DefaultWgpuGraphicalAdapterPipelineFactory {
            shader_code,
        }
    }
}

impl <'a> WgpuGraphicalAdapterPipelineFactory for DefaultWgpuGraphicalAdapterPipelineFactory<'a> {
    fn create(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> Box<dyn WgpuGraphicalAdapterPipeline> {
        Box::new(DefaultWgpuGraphicalAdapterPipeline::new(device, config, self.shader_code))
    }
}

pub struct DefaultWgpuGraphicalAdapterPipeline {
    pipeline: wgpu::RenderPipeline,
}

impl DefaultWgpuGraphicalAdapterPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        shader_code: &str,
    ) -> DefaultWgpuGraphicalAdapterPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(shader_code.into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        DefaultWgpuGraphicalAdapterPipeline {
            pipeline: render_pipeline,
        }
    }
}

impl WgpuGraphicalAdapterPipeline for DefaultWgpuGraphicalAdapterPipeline {
    fn get_inner(&self) -> &wgpu::RenderPipeline {
        &self.pipeline
    }
}