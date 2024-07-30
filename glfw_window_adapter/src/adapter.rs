use anyhow::Context;
use common::ScreenSize;

pub struct GLFWAdapter {
    glfw: glfw::Glfw,
    window: std::sync::Arc<glfw::PWindow>,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>
}

impl GLFWAdapter {
    pub fn new() -> anyhow::Result<GLFWAdapter> {
        use glfw::fail_on_errors;
        log::info!("Initializing GLFW.");

        let mut glfw = glfw::init(fail_on_errors!())
            .context("Failed to initialize GLFW.")?;

        log::info!("GLFW initialized.");

        // Disable default OpenGL
        glfw.window_hint(glfw::WindowHint::ClientApi(glfw::ClientApiHint::NoApi));

        log::info!("Creating GLFW window.");

        let (mut window, events) = glfw
            .create_window(800, 600, "Hello this is window", glfw::WindowMode::Windowed)
            .context("Failed to create GLFW window.")?;

        log::info!("GLFW window created.");

        window.set_key_polling(true);
        
        Ok(GLFWAdapter {
            glfw,
            window: std::sync::Arc::new(window),
            events,
        })
    }

    pub fn poll_events<F>(&mut self, handle_event: F) where F: Fn(&glfw::Window, glfw::WindowEvent) {
        self.glfw.poll_events();
        for (_, event) in glfw::flush_messages(&self.events) {
            handle_event(&self.window, event);
        }
    }

    pub fn should_loop_continue(&mut self) -> bool {
        !self.window.should_close()
    }

    pub fn get_window(&self) -> std::sync::Arc<glfw::PWindow> {
        self.window.clone()
    }

    pub fn get_window_size(&self) -> ScreenSize {
        ScreenSize {
            width: self.window.get_size().0 as _,
            height: self.window.get_size().1 as _,
        }
    }
}