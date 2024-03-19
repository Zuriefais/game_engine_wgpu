use glam::Mat4;

pub struct Camera {
    pub uniform: CameraUniform,
    pub right: f32,
    pub left: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(right: f32, left: f32, top: f32, bottom: f32, near: f32, far: f32) -> Self {
        Self {
            uniform: Camera::create_matrix(right, left, top, bottom, near, far),
            right,
            left,
            top,
            bottom,
            near,
            far,
        }
    }

    pub fn create_matrix(
        right: f32,
        left: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
    ) -> CameraUniform {
        CameraUniform {
            view_proj: Mat4::orthographic_rh(left, right, bottom, top, near, far)
                .to_cols_array_2d(),
        }
    }

    pub fn create_camera_from_screen_size(width: f32, height: f32, near: f32, far: f32) -> Camera {
        let aspect = width / height;
        let left = -aspect / 2.0;
        let right = aspect / 2.0;
        let bottom = -0.5;
        let top = 0.5;
        return Camera {
            uniform: CameraUniform {
                view_proj: Mat4::orthographic_rh(left, right, bottom, top, near, far)
                    .to_cols_array_2d(),
            },
            right,
            left,
            top,
            bottom,
            near,
            far,
        };
    }
}
