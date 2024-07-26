use anyhow::Context;

pub struct GLFWWrapper {
    glfw: glfw::Glfw,
    window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>
}

impl GLFWWrapper {
    pub fn new() -> anyhow::Result<GLFWWrapper> {
        use glfw::fail_on_errors;
        let mut glfw = glfw::init(fail_on_errors!())
            .context("Failed to initialize GLFW.")?;

        // Disable default OpenGL
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        let (mut window, events) = glfw
            .create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window.")?;

        window.set_key_polling(true);

        Ok(GLFWWrapper {
            glfw,
            window,
            events
        })
    }

    pub fn poll_events<F>(&mut self, handle_event: F) where F: Fn(&mut glfw::Window, glfw::WindowEvent) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            handle_event(&mut self.window, event);
        }
    }
    
    pub fn should_loop_continue(&mut self) -> bool {
        !self.window.should_close()
    }
}