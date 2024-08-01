use anyhow::Context;
use common::gateways::ImageLoaderGatewayResult;
use crate::camera::PerspectiveCamera;
use crate::instance::Instance;
use crate::pipeline::{WgpuGraphicalAdapterPipeline, WgpuGraphicalAdapterPipelineFactory};
use crate::vertex::Vertex;

pub struct WgpuGraphicalAdapterState<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: common::ScreenSize,
    pub camera: PerspectiveCamera,
    pub render_pipeline: Box<dyn WgpuGraphicalAdapterPipeline>,
}

impl<'a> WgpuGraphicalAdapterState<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(
        window: wgpu::SurfaceTarget<'a>,
        size: common::ScreenSize,
        factory: Box<dyn WgpuGraphicalAdapterPipelineFactory>,
    ) -> anyhow::Result<WgpuGraphicalAdapterState<'a>> {
        Self::validate_size(&size)?;

        let instance = Self::initialize_instance();
        let surface = Self::create_surface(window, &instance)?;
        let adapter = Self::request_adapter(instance, &surface).await?;
        let (device, queue) = Self::request_device_and_queue(&adapter).await?;
        let config = Self::configure_surface(&size, &surface, &adapter, &device);

        let camera = PerspectiveCamera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: (0.0, 1.0, 2.0).into(),
            // have it look at the origin
            target: (0.0, 0.0, 0.0).into(),
            // which way is "up"
            up: cgmath::Vector3::unit_y(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,
        };

        let render_pipeline = factory.create(&device, &config, &camera);
        

        Ok(WgpuGraphicalAdapterState {
            surface,
            device,
            queue,
            config,
            size,
            camera,
            render_pipeline,
        })
    }

    pub fn load_model_sync(&mut self,
                           vertices: Vec<Vertex>,
                           indices: Vec<u16>,
                           instances: Vec<Instance>,
                           raw_texture_data: ImageLoaderGatewayResult,
    ) -> anyhow::Result<()> {
        self.render_pipeline.load_model_sync(
            vertices,
            indices,
            instances,
            raw_texture_data,
            &self.device,
            &self.queue
        )
    }
    
    pub fn update_camera(&mut self) {
        self.render_pipeline.update_camera(&self.camera, &self.queue);
    }
    
    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.render_pipeline.render(&mut render_pass);
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn validate_size(size: &common::ScreenSize) -> anyhow::Result<()> {
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

    fn create_surface(window: wgpu::SurfaceTarget<'a>, instance: &wgpu::Instance) -> anyhow::Result<wgpu::Surface<'a>> {
        log::info!("Creating surface...");
        let surface = instance.create_surface(window)?;
        log::info!("Finished creating surface.");
        Ok(surface)
    }

    async fn request_adapter(instance: wgpu::Instance, surface: &wgpu::Surface<'a>) -> anyhow::Result<wgpu::Adapter> {
        log::info!("Requesting adapter...");
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
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

    fn configure_surface(
        size: &common::ScreenSize,
        surface: &wgpu::Surface,
        adapter: &wgpu::Adapter,
        device: &wgpu::Device
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