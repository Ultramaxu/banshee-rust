#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct PerspectiveCamera {
    pub eye: cgmath::Point3<f32>, // By eye we mean the position of the camera.
    pub target: cgmath::Point3<f32>, // By target we mean the point the camera is looking at.
    pub up: cgmath::Vector3<f32>, // By up we mean the direction that is up for the camera.
    pub aspect: f32, // The aspect ratio of the camera.
    pub fovy: f32, // The field of view of the camera in radians.
    pub znear: f32, // The near clipping plane of the camera.
    pub zfar: f32,  // The far clipping plane of the camera.
}

impl PerspectiveCamera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        return OPENGL_TO_WGPU_MATRIX * proj * view;
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    pub view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &PerspectiveCamera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
 