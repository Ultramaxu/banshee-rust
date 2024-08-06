use std::time::{Instant, SystemTime};

use crate::utils::{CUBE_MODEL_ID, DEFAULT_PIPELINE_ID, get_cube_instances_by_absolute_time, load_cube_for_default_pipeline, make_adapter_with_glfw_window, make_glfw_adapter};

pub fn run() -> anyhow::Result<()> {
    let mut glfw_adapter = make_glfw_adapter("Simple Cube")?;
    let mut state = make_adapter_with_glfw_window(&glfw_adapter)?;
    load_cube_for_default_pipeline(&mut state, get_cube_instances_by_absolute_time(0))?;

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
        let forward = state.get_camera().target - state.get_camera().eye;
        let forward_norm = forward.normalize();
        let forward_mag = forward.magnitude();
        let right = forward_norm.cross(state.get_camera().up);
        state.update_camera_eye(state.get_camera().target - (forward + right * 0.02).normalize() * forward_mag);

        state.update_model_instances(
            DEFAULT_PIPELINE_ID,
            CUBE_MODEL_ID,
            get_cube_instances_by_absolute_time(acc_time),
        )?;

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

    Ok(())
}