use std::time::Instant;

use cgmath::prelude::*;
use pollster::FutureExt as _;

use common::gateways::ImageLoaderGateway;
use glfw_window_adapter::adapter::GLFWAdapter;
use image_crate_image_loader_adapter::ImageCrateImageLoaderAdapter;
use wgpu_graphical_adapter::default_pipeline_impl::default_pipeline::DefaultWgpuGraphicalAdapterPipelineFactory;
use wgpu_graphical_adapter::instance::Instance;
use wgpu_graphical_adapter::state::WgpuGraphicalAdapterState;
use wgpu_graphical_adapter::vertex::Vertex;

fn main() {
    structured_logger::Builder::with_level("info")
        .with_target_writer("*", structured_logger::json::new_writer(std::io::stdout()))
        .init();
    let mut glfw_adapter = match GLFWAdapter::new("Banshee Engine v0.0.0 - GLFW/WGPU - Desktop Target - Test") {
        Ok(glfw_adapter) => glfw_adapter,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };

    let image_loader_gateway = ImageCrateImageLoaderAdapter::new();
    let mut state = match WgpuGraphicalAdapterState::new(
        glfw_adapter.get_window().into(),
        glfw_adapter.get_window_size(),
        Box::new(DefaultWgpuGraphicalAdapterPipelineFactory::new()),
    ).block_on() {
        Ok(state) => state,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);

    let model_vertices = vec![
        Vertex { position: [-0.0868241, 0.49240386, 0.0], tex_coords: [0.4131759, 0.00759614] }, // A
        Vertex { position: [-0.49513406, 0.06958647, 0.0], tex_coords: [0.0048659444, 0.43041354] }, // B
        Vertex { position: [-0.21918549, -0.44939706, 0.0], tex_coords: [0.28081453, 0.949397] }, // C
        Vertex { position: [0.35966998, -0.3473291, 0.0], tex_coords: [0.85967, 0.84732914] }, // D
        Vertex { position: [0.44147372, 0.2347359, 0.0], tex_coords: [0.9414737, 0.2652641] }, // E
    ];
    let model_indices = vec![
        0, 1, 4,
        1, 2, 4,
        2, 3, 4,
    ];
    let model_instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - INSTANCE_DISPLACEMENT;

            let rotation = if position.is_zero() {
                // this is needed so an object at (0, 0, 0) won't get scaled to zero
                // as Quaternions can affect scale if they're not created correctly
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
            };

            Instance { position, rotation }
        })
    }).collect::<Vec<_>>();
    let texture_id = "texture.png";
    
    state.load_model_sync(
        model_vertices,
        model_indices,
        model_instances,
        image_loader_gateway.load_sync(texture_id).unwrap(),
    ).unwrap();

    while glfw_adapter.should_loop_continue() {

        use cgmath::InnerSpace;
        let forward = state.camera.target - state.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let right = forward_norm.cross(state.camera.up);
        state.camera.eye = state.camera.target - (forward + right * 0.02).normalize() * forward_mag;
        state.update_camera();
        let start = Instant::now();
        state.render().unwrap();
        let elapsed = start.elapsed();
        log::info!(
            target: "performance",
            time_unit = "microseconds",
            frame_time = elapsed.as_micros();
            "",
        );

        glfw_adapter.poll_events(|_, event| {
            log::info!("{:?}", event);
        });
    }
}
