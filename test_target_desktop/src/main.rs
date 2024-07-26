use glfw_window_adapter::primary_adapter::GLFWPrimaryAdapter;

fn main() {
    structured_logger::Builder::with_level("info")
        .with_target_writer("*", structured_logger::json::new_writer(std::io::stdout()))
        .init();
    let mut glfw_adapter = match GLFWPrimaryAdapter::new() {
        Ok(glfw_adapter) => glfw_adapter,
        Err(e) => {
            log::error!("{:?}", e);
            return;
        }
    };

    while glfw_adapter.should_loop_continue() {
        glfw_adapter.poll_events(|window, event| {
            log::debug!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        });
    }
}
