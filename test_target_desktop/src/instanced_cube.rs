use std::path::Path;
use crate::utils::{get_cube_instances_by_absolute_time, load_cube_for_default_pipeline, make_adapter_to_render_to_disk};

pub fn run(output_path: &Path) -> anyhow::Result<()> {
    let mut state = make_adapter_to_render_to_disk(
        output_path.join("instanced_cube.png").into_boxed_path()
    )?;
    load_cube_for_default_pipeline(&mut state, get_cube_instances_by_absolute_time(0))?;
    state.render().unwrap();
    Ok(())
}