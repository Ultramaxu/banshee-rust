use std::path::Path;

use cgmath::Rotation3;

use wgpu_graphical_adapter::instance::Instance;

use crate::utils::{load_cube_for_default_pipeline, make_adapter_to_render_to_disk};

pub fn run(output_path: &Path) -> anyhow::Result<()> {
    let mut state = make_adapter_to_render_to_disk(
        output_path.join("simple_cube.png").into_boxed_path()
    )?;
    load_cube_for_default_pipeline(&mut state, vec![Instance {
        position: cgmath::Vector3::new(0.0, 0.0, 0.0),
        rotation: cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0)),
    }])?;
    state.render().unwrap();
    Ok(())
}