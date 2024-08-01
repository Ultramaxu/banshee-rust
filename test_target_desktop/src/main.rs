use std::rc::Rc;
use std::time::Instant;

use cgmath::prelude::*;
use pollster::FutureExt as _;


use glfw_window_adapter::adapter::GLFWAdapter;
use wgpu_graphical_adapter::default_pipeline_impl::default_pipeline::DefaultWgpuGraphicalAdapterPipelineFactory;
use wgpu_graphical_adapter::instance::Instance;
use wgpu_graphical_adapter::state::WgpuGraphicalAdapterState;
use wgpu_obj_model_loader_adapter::ObjWgpuModelLoaderAdapter;

fn main() {
    structured_logger::Builder::new()
        .with_target_writer("*", structured_logger::json::new_writer(std::io::stdout()))
        .init();
    let mut glfw_adapter = match GLFWAdapter::new("Banshee Engine v0.0.0 - GLFW/WGPU - Desktop Target - Test") {
        Ok(glfw_adapter) => glfw_adapter,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };
    let wgpu_obj_model_loader_adapter = Rc::new(ObjWgpuModelLoaderAdapter::new(
        Box::from(env!("OUT_DIR")),
    ));
    let mut state = match WgpuGraphicalAdapterState::new(
        glfw_adapter.get_window().into(),
        glfw_adapter.get_window_size(),
        Box::new(DefaultWgpuGraphicalAdapterPipelineFactory::new(wgpu_obj_model_loader_adapter)),
    ).block_on() {
        Ok(state) => state,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };

    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;
    let model_instances = (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
            let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

            let position = cgmath::Vector3 { x, y: 0.0, z };

            let rotation = if position.is_zero() {
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
            };

            Instance {
                position, rotation,
            }
        })
    }).collect::<Vec<_>>();
    let model_filename = "cube.obj";

    match state.load_model_sync(
        model_filename,
        model_instances,
    ) {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    }

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
