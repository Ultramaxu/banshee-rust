use glfw_window_adapter::GLFWWrapper;

fn main() {
    let mut glfw_wrapper = match GLFWWrapper::new() { 
        Ok(glfw_wrapper) => glfw_wrapper,
        Err(e) => {
            eprintln!("{:?}", e);
            return;
        }
    };
    
    while glfw_wrapper.should_loop_continue() {
        glfw_wrapper.poll_events(|window, event| {
            println!("{:?}", event);
            match event {
                glfw::WindowEvent::Key(glfw::Key::Escape, _, glfw::Action::Press, _) => {
                    window.set_should_close(true)
                }
                _ => {}
            }
        });
    }
}
