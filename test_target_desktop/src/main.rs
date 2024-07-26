use glfw::PWindow;
use pollster::FutureExt as _;
use glfw_window_adapter::adapter::GLFWAdapter;
use wgpu_graphical_adapter::state::{State};

fn main() {
    structured_logger::Builder::with_level("info")
        .with_target_writer("*", structured_logger::json::new_writer(std::io::stdout()))
        .init();
    let mut glfw_adapter = match GLFWAdapter::new() {
        Ok(glfw_adapter) => glfw_adapter,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };
    
    let mut state = match State::new(
        glfw_adapter.get_window().into(),
        glfw_adapter.get_window_size(),
    ).block_on() {
        Ok(state) => state,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };

    while glfw_adapter.should_loop_continue() {
        glfw_adapter.poll_events(|_, event| {
            log::info!("{:?}", event);
        });
    }
}
