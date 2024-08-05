use std::collections::HashMap;
use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::camera::{CameraUniform, PerspectiveCamera};
use crate::gateways::WgpuModelLoaderGateway;
use crate::instance::{Instance, InstanceRaw};
use crate::model::Model;
use crate::pipeline::{WgpuGraphicalAdapterPipeline, WgpuGraphicalAdapterPipelineFactory};
use crate::texture::Texture;
use crate::vertex::ModelVertex;

pub struct DefaultWgpuGraphicalAdapterPipelineFactory {
    model_loader_gateway: Rc<dyn WgpuModelLoaderGateway>,
}

impl DefaultWgpuGraphicalAdapterPipelineFactory {
    pub fn new(
        model_loader_gateway: Rc<dyn WgpuModelLoaderGateway>
    ) -> DefaultWgpuGraphicalAdapterPipelineFactory {
        DefaultWgpuGraphicalAdapterPipelineFactory {
            model_loader_gateway,
        }
    }
}

impl WgpuGraphicalAdapterPipelineFactory for DefaultWgpuGraphicalAdapterPipelineFactory {
    fn create(
        &self,
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera: &PerspectiveCamera,
    ) -> Box<dyn WgpuGraphicalAdapterPipeline> {
        Box::new(DefaultWgpuGraphicalAdapterPipeline::new(
            device,
            config,
            camera,
            self.model_loader_gateway.clone(),
        ))
    }
}

pub struct DefaultWgpuGraphicalAdapterPipeline {
    model_loader_gateway: Rc<dyn WgpuModelLoaderGateway>,
    pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    models: HashMap<String, Model>,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
}

impl DefaultWgpuGraphicalAdapterPipeline {
    pub fn new(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        camera: &PerspectiveCamera,
        model_loader_gateway: Rc<dyn WgpuModelLoaderGateway>,
    ) -> DefaultWgpuGraphicalAdapterPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Default Render Pipeline Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("Default Render Pipeline Texture Bind Group Layout"),
            });


        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Default Pipeline Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("Default Pipeline Camera Bind Group Layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("Default Pipeline Camera Bind Group"),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Default Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[
                    DefaultWgpuGraphicalAdapterPipeline::get_model_vertex_desc(),
                    DefaultWgpuGraphicalAdapterPipeline::get_instance_raw_desc(),
                ],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // draw pixels from front to back
                stencil: wgpu::StencilState::default(), // TODO later
                bias: wgpu::DepthBiasState::default(),
            }),
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
            texture_bind_group_layout,
            models: HashMap::new(),
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            model_loader_gateway,
        }
    }

    fn get_model_vertex_desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<ModelVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    fn get_instance_raw_desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl WgpuGraphicalAdapterPipeline for DefaultWgpuGraphicalAdapterPipeline {
    fn load_model_sync(
        &mut self,
        id: &str,
        filename: &str,
        instances: Vec<Instance>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> anyhow::Result<()> {
        let model = self.model_loader_gateway.load_model_sync(
            filename,
            instances,
            device,
            queue,
            &self.texture_bind_group_layout,
            Box::new(|
                device,
                texture_view,
                sampler,
                layout
            | {
                device.create_bind_group(
                    &wgpu::BindGroupDescriptor {
                        layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(&texture_view),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(&sampler),
                            }
                        ],
                        label: None,
                    }
                )
            }),
        )?;
        self.models.insert(id.to_string(), model);
        Ok(())
    }

    fn update_camera(&mut self, camera: &PerspectiveCamera, queue: &wgpu::Queue) {
        self.camera_uniform.update_view_proj(camera);
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[self.camera_uniform]));
    }

    fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        render_pass.set_pipeline(&self.pipeline);
        for (id, model) in &self.models {

            log::debug!("Drawing model: {}", id);
            use crate::model::DrawModel;
            render_pass.draw_model_instanced(model, &self.camera_bind_group, None);
        }
    }
    
    fn update_model_instances(&mut self, model_id: &str, instances: Vec<Instance>, device: &wgpu::Device) -> anyhow::Result<()> {
        if let Some(model) = self.models.get_mut(model_id) {
            model.instances = Instance::instances_to_buffer(&instances, device);
            Ok(())
        } else {
            Err(anyhow::anyhow!("Model not found: {}", model_id))
        }
    }
}