use anyhow::Context;
use wgpu::{MemoryHints, PresentMode, SurfaceTarget};
use common::common_defs::ScreenSize;

pub struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: ScreenSize,
}

impl<'a> State<'a> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: SurfaceTarget<'a>, size: ScreenSize) -> anyhow::Result<State<'a>> {
        if (size.width == 0) || (size.height == 0) {
            return Err(anyhow::anyhow!("Invalid screen size: width: {}, height: {}", size.width, size.height));
        }
        
        log::info!("Initializing wgpu...");
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });
        log::info!("Finished initializing wgpu.");
        
        log::info!("Creating surface...");
        let surface = instance.create_surface(window)?;
        log::info!("Finished creating surface.");

        log::info!("Requesting adapter...");
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false, // we only want a hardware adapter
            },
        ).await.context("Unable to request WGPU adapter")?;
        log::info!("Finished requesting adapter.");

        log::info!("Requesting device and queue...");
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                label: None,
                memory_hints: MemoryHints::Performance,
            },
            None, // Trace path
        ).await.context("Unable to request WGPU device and queue")?;
        log::info!("Finished requesting device and queue.");

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
            present_mode: PresentMode::Fifo, // vsync, will always be supported.
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        log::info!("Finished configuring surface...");


        Ok(State {
            surface,
            device,
            queue,
            config,
            size,
        })
    }
    
    pub fn render(&mut self) -> anyhow::Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
        }

        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}