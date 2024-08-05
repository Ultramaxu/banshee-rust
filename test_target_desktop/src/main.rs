use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Instant, SystemTime};

use cgmath::num_traits::real::Real;
use cgmath::prelude::*;
use pollster::FutureExt as _;

use glfw_window_adapter::adapter::GLFWAdapter;
use wgpu_graphical_adapter::default_pipeline_impl::default_pipeline::DefaultWgpuGraphicalAdapterPipelineFactory;
use wgpu_graphical_adapter::instance::Instance;
use wgpu_graphical_adapter::pipeline::WgpuGraphicalAdapterPipelineFactory;
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
    let DEFAULT_PIPELINE_ID = "default";
    let CUBE_MODEL_ID = "cube_1";
    let mut factories: HashMap<String, Box<dyn WgpuGraphicalAdapterPipelineFactory>> = HashMap::new();
    factories.insert(
        DEFAULT_PIPELINE_ID.to_string(),
        Box::new(DefaultWgpuGraphicalAdapterPipelineFactory::new(wgpu_obj_model_loader_adapter)),
    );
    let mut state = match WgpuGraphicalAdapterState::new(
        glfw_adapter.get_window().into(),
        glfw_adapter.get_window_size(),
        factories,
    ).block_on() {
        Ok(state) => state,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };
    let model_instances = get_cube_instances_by_absolute_time(0);
    let model_filename = "cube.obj";

    match state.load_model_sync(
        DEFAULT_PIPELINE_ID,
        CUBE_MODEL_ID,
        model_filename,
        model_instances,
    ) {
        Ok(_) => {}
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    }

    let mut time = SystemTime::now();
    let mut acc_time: u32 = 0;
    while glfw_adapter.should_loop_continue() {
        let delta = time.elapsed().unwrap().as_millis() as u32;
        acc_time += delta;
        time = SystemTime::now();
        glfw_adapter.poll_events(|_, event| {
            log::info!("{:?}", event);
        });

        use cgmath::InnerSpace;
        let forward = state.camera.target - state.camera.eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let right = forward_norm.cross(state.camera.up);
        state.camera.eye = state.camera.target - (forward + right * 0.02).normalize() * forward_mag;
        state.update_camera();

        match state.update_model_instances(
            DEFAULT_PIPELINE_ID,
            CUBE_MODEL_ID,
            get_cube_instances_by_absolute_time(acc_time),
        ) {
            Ok(_) => {}
            Err(e) => {
                log::error!("{:?}", e);
                return;
            }
        }

        let start = Instant::now();
        state.render().unwrap();
        let elapsed = start.elapsed();
        log::info!(
            target: "performance",
            time_unit = "microseconds",
            frame_time = elapsed.as_micros();
            "",
        );
    }
}

fn get_cube_instances_by_absolute_time(time: u32) -> Vec<Instance> {
    const NUM_INSTANCES_PER_ROW: u32 = 10;
    const SPACE_BETWEEN: f32 = 3.0;

    (0..NUM_INSTANCES_PER_ROW).flat_map(|z| {
        (0..NUM_INSTANCES_PER_ROW).map(move |x| {
            let x = SPACE_BETWEEN * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
            let y = ((time as f32 / 1000.0) + z as f32).sin();
            let z = SPACE_BETWEEN * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);

            let position_for_rotation = cgmath::Vector3 { x, y: 0.0, z };

            let rotation = if position_for_rotation.is_zero() {
                cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
            } else {
                cgmath::Quaternion::from_axis_angle(position_for_rotation.normalize(), cgmath::Deg(45.0))
            };

            Instance {
                position: cgmath::Vector3 { x, y, z },
                rotation,
            }
        })
    }).collect::<Vec<_>>()
}
