use glam::{Mat4, Vec2};

pub struct Camera {
    pub uniform: CameraUniform,
    pub right: f32,
    pub left: f32,
    pub top: f32,
    pub bottom: f32,
    pub near: f32,
    pub far: f32,
    pub zoom_factor: f32,
    pub position: Vec2,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4],
}

impl Camera {
    pub fn new(
        right: f32,
        left: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
        zoom_factor: f32,
        position: Vec2,
    ) -> Self {
        Self {
            uniform: Camera::create_matrix(right, left, top, bottom, near, far, zoom_factor),
            right,
            left,
            top,
            bottom,
            near,
            far,
            zoom_factor,
            position,
        }
    }

    pub fn create_matrix(
        right: f32,
        left: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
        zoom_factor: f32,
    ) -> CameraUniform {
        CameraUniform {
            view_proj: Mat4::orthographic_rh(
                left + left * zoom_factor,
                right + right * zoom_factor,
                bottom + bottom * zoom_factor,
                top + top * zoom_factor,
                near,
                far,
            )
            .to_cols_array_2d(),
        }
    }

    pub fn create_camera_from_screen_size(
        width: f32,
        height: f32,
        near: f32,
        far: f32,
        zoom_factor: f32,
        position: Vec2,
    ) -> Camera {
        let aspect = width / height;
        let left = -aspect / 2.0;
        let right = aspect / 2.0;
        let bottom = -0.5;
        let top = 0.5;
        Camera::new(right, left, top, bottom, near, far, zoom_factor, position)
    }

    pub fn update_matrix(&mut self) {
        self.uniform = Camera::create_matrix(
            self.right,
            self.left,
            self.top,
            self.bottom,
            self.near,
            self.far,
            self.zoom_factor,
        )
    }
}
