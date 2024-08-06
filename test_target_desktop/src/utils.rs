use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

use cgmath::{InnerSpace, Zero};
use cgmath::prelude::*;
use pollster::FutureExt;

use glfw_window_adapter::adapter::GLFWAdapter;
use wgpu_graphical_adapter::default_pipeline_impl::default_pipeline::DefaultWgpuGraphicalAdapterPipelineFactory;
use wgpu_graphical_adapter::instance::Instance;
use wgpu_graphical_adapter::pipeline::WgpuGraphicalAdapterPipelineFactory;
use wgpu_graphical_adapter::state::{WgpuGraphicalAdapterState, WgpuGraphicalAdapterStateRenderToDisk, WgpuGraphicalAdapterStateWithWindow};
use wgpu_obj_model_loader_adapter::ObjWgpuModelLoaderAdapter;

pub static DEFAULT_PIPELINE_ID: &'static str = "default";
pub static CUBE_MODEL_ID: &'static str = "cube_1";

pub fn make_glfw_adapter(test_name: &str) -> anyhow::Result<GLFWAdapter> {
    let title = format!("Banshee Engine v0.0.0 - GLFW/WGPU - Desktop Target - {}", test_name);
    GLFWAdapter::new(title.as_str())
}

pub fn make_adapter_with_glfw_window(
    glfw_adapter: &GLFWAdapter
) -> anyhow::Result<Box<dyn WgpuGraphicalAdapterState>> {
    let wgpu_obj_model_loader_adapter = Rc::new(ObjWgpuModelLoaderAdapter::new(
        Box::from(env!("OUT_DIR")),
    ));
    let mut factories: HashMap<String, Box<dyn WgpuGraphicalAdapterPipelineFactory>> = HashMap::new();
    factories.insert(
        DEFAULT_PIPELINE_ID.to_string(),
        Box::new(DefaultWgpuGraphicalAdapterPipelineFactory::new(wgpu_obj_model_loader_adapter)),
    );
    Ok(Box::new(WgpuGraphicalAdapterStateWithWindow::new(
        glfw_adapter.get_window().into(),
        glfw_adapter.get_window_size(),
        factories,
    ).block_on()?))
}

pub fn make_adapter_to_render_to_disk(output_path: Box<Path>) -> anyhow::Result<Box<dyn WgpuGraphicalAdapterState>> {
    let wgpu_obj_model_loader_adapter = Rc::new(ObjWgpuModelLoaderAdapter::new(
        Box::from(env!("OUT_DIR")),
    ));
    let mut factories: HashMap<String, Box<dyn WgpuGraphicalAdapterPipelineFactory>> = HashMap::new();
    factories.insert(
        DEFAULT_PIPELINE_ID.to_string(),
        Box::new(DefaultWgpuGraphicalAdapterPipelineFactory::new(wgpu_obj_model_loader_adapter)),
    );
    Ok(Box::new(WgpuGraphicalAdapterStateRenderToDisk::new(
        common::Dimentions {
            width: 512,
            height: 512,
        },
        factories,
        output_path,
    ).block_on()?))

    
}

pub fn load_cube_for_default_pipeline(
    state: &mut Box<dyn WgpuGraphicalAdapterState>,
    model_instances: Vec<Instance>,
) -> anyhow::Result<()> {
    let model_filename = "cube.obj";
    state.load_model_sync(
        DEFAULT_PIPELINE_ID,
        CUBE_MODEL_ID,
        model_filename,
        model_instances,
    )
}

pub fn get_cube_instances_by_absolute_time(time: u32) -> Vec<Instance> {
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