use std::collections::HashMap;
use std::path::Path;
use anyhow::Context;
use pollster::FutureExt;
use crate::camera::PerspectiveCamera;
use crate::instance::Instance;
use crate::pipeline::{WgpuGraphicalAdapterPipeline, WgpuGraphicalAdapterPipelineFactory};
use crate::texture::{RenderTargetTexture, Texture};

pub trait WgpuGraphicalAdapterState {
    fn load_model_sync(&mut self,
                       pipeline_id: &str,
                       model_id: &str,
                       filename: &str,
                       instances: Vec<Instance>,
    ) -> anyhow::Result<()>;
    fn get_camera(&self) -> &PerspectiveCamera;
    fn update_camera_eye(&mut self, eye: cgmath::Point3<f32>);
    fn update_model_instances(
        &mut self,
        pipeline_id: &str,
        model_id: &str,
        instances: Vec<Instance>
    ) -> anyhow::Result<()>;
    fn render(&mut self) -> anyhow::Result<()>;
}

pub struct CoreState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_texture: Texture,
    pub camera: PerspectiveCamera,
    pub render_pipelines: HashMap<String, Box<dyn WgpuGraphicalAdapterPipeline>>,
}

impl<'a> CoreState {
    pub fn load_model_sync(&mut self,
                           pipeline_id: &str,
                           model_id: &str,
                           filename: &str,
                           instances: Vec<Instance>,
    ) -> anyhow::Result<()> {
        if let Some(pipeline) = self.render_pipelines.get_mut(pipeline_id) {
            pipeline.load_model_sync(model_id, filename, instances, &self.device, &self.queue)
        } else {
            Err(anyhow::anyhow!("Pipeline not found: {}", pipeline_id))
        }
    }

    pub fn update_camera(&mut self) {
        for (_, pipeline) in self.render_pipelines.iter_mut() {
            pipeline.update_camera(&self.camera, &self.queue);
        }
    }

    pub fn update_model_instances(&mut self, pipeline_id: &str, model_id: &str, instances: Vec<Instance>) -> anyhow::Result<()> {
        if let Some(pipeline) = self.render_pipelines.get_mut(pipeline_id) {
            pipeline.update_model_instances(model_id, instances, &self.device)
        } else {
            Err(anyhow::anyhow!("Pipeline not found: {}", pipeline_id))
        }
    }

    fn new(
        device: wgpu::Device,
        queue: wgpu::Queue,
        depth_texture: Texture,
        camera: PerspectiveCamera,
        render_pipelines: HashMap<String, Box<dyn WgpuGraphicalAdapterPipeline>>,
    ) -> CoreState {
        CoreState {
            device,
            queue,
            depth_texture,
            camera,
            render_pipelines,
        }
    }

    fn validate_size(size: &common::Dimentions) -> anyhow::Result<()> {
        if (size.width == 0) || (size.height == 0) {
            return Err(anyhow::anyhow!("Invalid screen size: width: {}, height: {}", size.width, size.height));
        }
        Ok(())
    }

    fn initialize_instance() -> wgpu::Instance {
        log::info!("Initializing wgpu...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        log::info!("Finished initializing wgpu.");
        instance
    }

    async fn request_adapter(instance: wgpu::Instance, surface: Option<&wgpu::Surface<'a>>) -> anyhow::Result<wgpu::Adapter> {
        log::info!("Requesting adapter...");
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: surface,
                force_fallback_adapter: false, // we only want a hardware adapter
            },
        ).await.context("Unable to request WGPU adapter")?;
        log::info!("Finished requesting adapter.");
        Ok(adapter)
    }

    async fn request_device_and_queue(adapter: &wgpu::Adapter) -> anyhow::Result<(wgpu::Device, wgpu::Queue)> {
        log::info!("Requesting device and queue...");
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None, // Trace path
        ).await.context("Unable to request WGPU device and queue")?;
        log::info!("Finished requesting device and queue.");
        Ok((device, queue))
    }

    fn get_render_pass_stencil_attachment(&self) -> wgpu::RenderPassDepthStencilAttachment {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }
}

pub struct WgpuGraphicalAdapterStateWithWindow<'a> {
    core_state: CoreState,
    surface: wgpu::Surface<'a>,
}

impl<'a> WgpuGraphicalAdapterStateWithWindow<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(
        window: wgpu::SurfaceTarget<'a>,
        size: common::Dimentions,
        factories: HashMap<String, Box<dyn WgpuGraphicalAdapterPipelineFactory>>,
    ) -> anyhow::Result<WgpuGraphicalAdapterStateWithWindow<'a>> {
        CoreState::validate_size(&size)?;

        let instance = CoreState::initialize_instance();
        let surface = Self::create_surface(window, &instance)?;
        let adapter = CoreState::request_adapter(instance, Some(&surface)).await?;
        let (device, queue) = CoreState::request_device_and_queue(&adapter).await?;
        let configuration = Self::configure_surface(&size, &surface, &adapter, &device);

        let depth_texture = Texture::new_depth_texture(
            &device,
            common::Dimentions {
                width: size.width,
                height: size.height,
            },
            "depth_texture",
        );
        let camera = make_camera(size);

        let mut render_pipelines: HashMap<String, Box<dyn WgpuGraphicalAdapterPipeline>> = HashMap::new();
        for (name, factory) in factories {
            render_pipelines.insert(name, factory.create(
                &device,
                configuration.format,
                &camera,
            ));
        }

        Ok(WgpuGraphicalAdapterStateWithWindow {
            core_state: CoreState::new(device, queue, depth_texture, camera, render_pipelines),
            surface,
        })
    }

    fn create_surface(window: wgpu::SurfaceTarget<'a>, instance: &wgpu::Instance) -> anyhow::Result<wgpu::Surface<'a>> {
        log::info!("Window provided, creating surface...");
        let surface = instance.create_surface(window)?;
        log::info!("Finished creating surface.");
        Ok(surface)
    }

    fn configure_surface(
        size: &common::Dimentions,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device,
    ) -> wgpu::SurfaceConfiguration {
        log::info!("Configuring surface...");
        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps.formats.iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // vsync, will always be supported.
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        log::info!("Finished configuring surface...");
        config
    }
}

impl <'a> WgpuGraphicalAdapterState for WgpuGraphicalAdapterStateWithWindow<'a> {
    fn load_model_sync(&mut self, pipeline_id: &str, model_id: &str, filename: &str, instances: Vec<Instance>) -> anyhow::Result<()> {
        self.core_state.load_model_sync(pipeline_id, model_id, filename, instances)
    }

    fn get_camera(&self) -> &PerspectiveCamera {
        &self.core_state.camera
    }

    fn update_camera_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.core_state.camera.eye = eye;
        self.core_state.update_camera();
    }

    fn update_model_instances(&mut self, pipeline_id: &str, model_id: &str, instances: Vec<Instance>) -> anyhow::Result<()> {
        self.core_state.update_model_instances(pipeline_id, model_id, instances)
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.core_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(self.core_state.get_render_pass_stencil_attachment()),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for (_, render_pipeline) in self.core_state.render_pipelines.iter() {
                render_pipeline.render(&mut render_pass);
            }
        }

        // submit will accept anything that implements IntoIter
        self.core_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub struct WgpuGraphicalAdapterStateRenderToDisk {
    core_state: CoreState,
    render_target_texture: RenderTargetTexture,
    output_path: Box<Path>,
}

impl WgpuGraphicalAdapterStateRenderToDisk {
    pub async fn new(
        size: common::Dimentions,
        factories: HashMap<String, Box<dyn WgpuGraphicalAdapterPipelineFactory>>,
        output_path: Box<Path>,
    ) -> anyhow::Result<WgpuGraphicalAdapterStateRenderToDisk> {
        CoreState::validate_size(&size)?;

        let instance = CoreState::initialize_instance();
        let adapter = CoreState::request_adapter(instance, None).await?;
        let (device, queue) = CoreState::request_device_and_queue(&adapter).await?;

        let render_target_texture = RenderTargetTexture::new(
            &device,
            &size,
            "render_target_texture"
        );
        let depth_texture = Texture::new_depth_texture(
            &device,
            common::Dimentions {
                width: size.width,
                height: size.height,
            },
            "depth_texture",
        );
        let camera = make_camera(size);

        let mut render_pipelines: HashMap<String, Box<dyn WgpuGraphicalAdapterPipeline>> = HashMap::new();
        for (name, factory) in factories {
            render_pipelines.insert(name, factory.create(
                &device,
                render_target_texture.texture.format(),
                &camera,
            ));
        }

        Ok(WgpuGraphicalAdapterStateRenderToDisk {
            core_state: CoreState::new(device, queue, depth_texture, camera, render_pipelines),
            render_target_texture,
            output_path,
        })
    }
}

impl WgpuGraphicalAdapterState for WgpuGraphicalAdapterStateRenderToDisk {
    fn load_model_sync(&mut self, pipeline_id: &str, model_id: &str, filename: &str, instances: Vec<Instance>) -> anyhow::Result<()> {
        self.core_state.load_model_sync(pipeline_id, model_id, filename, instances)
    }

    fn get_camera(&self) -> &PerspectiveCamera {
        &self.core_state.camera
    }

    fn update_camera_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.core_state.camera.eye = eye;
        self.core_state.update_camera();
    }

    fn update_model_instances(&mut self, pipeline_id: &str, model_id: &str, instances: Vec<Instance>) -> anyhow::Result<()> {
        self.core_state.update_model_instances(pipeline_id, model_id, instances)
    }

    fn render(&mut self) -> anyhow::Result<()> {
        let mut encoder = self.core_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(self.core_state.get_render_pass_stencil_attachment()),
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            for (_, render_pipeline) in self.core_state.render_pipelines.iter() {
                render_pipeline.render(&mut render_pass);
            }
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &self.render_target_texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            wgpu::ImageCopyBuffer {
                buffer: &self.render_target_texture.output_buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(size_of::<u32>() as u32
                        * self.render_target_texture.dimensions.width),
                    rows_per_image: Some(self.render_target_texture.dimensions.height),
                },
            },
            wgpu::Extent3d {
                width: self.render_target_texture.dimensions.width,
                height: self.render_target_texture.dimensions.height,
                depth_or_array_layers: 1,
            },
        );

        // submit will accept anything that implements IntoIter
        self.core_state.queue.submit(std::iter::once(encoder.finish()));

        // It's okay to block here, because rendering images to disk is not a performance-critical operation
        self.render_target_texture.to_file(
            &self.output_path,
            &self.core_state.device
        ).block_on()?;

        Ok(())
    }
}

fn make_camera(size: common::Dimentions) -> PerspectiveCamera {
    PerspectiveCamera {
        // position the camera 1 unit up and 2 units back
        // +z is out of the screen
        eye: (0.0, 6.0, 20.0).into(),
        // have it look at the origin
        target: (0.0, 0.0, 0.0).into(),
        // which way is "up"
        up: cgmath::Vector3::unit_y(),
        aspect: size.width as f32 / size.height as f32,
        fovy: 45.0,
        znear: 0.1,
        zfar: 100.0,
    }
}