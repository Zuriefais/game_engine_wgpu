use glam::{Mat4, Vec2, Vec4};

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
    pub position: Vec4,
    //pub position: [f32; 2],
}

impl Camera {
    pub fn new(
        right: f32,
        left: f32,
        top: f32,
        bottom: f32,
        near: f32,
        far: f32,
        mut zoom_factor: f32,
        position: Vec2,
    ) -> Self {
        if zoom_factor == 0.0 {
            zoom_factor = 1.0;
        }

        Self {
            uniform: CameraUniform {
                view_proj: Mat4::ZERO.to_cols_array_2d(),
                position: ((position / zoom_factor).extend(0f32).extend(0f32)),
            },
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

    pub fn create_matrix(&self) -> Mat4 {
        let mut zoom_factor = self.zoom_factor;
        if zoom_factor == 0.0 {
            zoom_factor = 1.0;
        }

        let adjusted_left = self.left + (self.left * zoom_factor);
        let adjusted_right = self.right + (self.right * zoom_factor);
        let adjusted_bottom = self.bottom + (self.bottom * zoom_factor);
        let adjusted_top = self.top + (self.top * zoom_factor);

        let projection_matrix = Mat4::orthographic_lh(
            adjusted_left,
            adjusted_right,
            adjusted_bottom,
            adjusted_top,
            self.near,
            self.far,
        );

        projection_matrix //* transform_matrix
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
        self.uniform = CameraUniform {
            view_proj: self.create_matrix().to_cols_array_2d(),
            position: ((self.position / self.zoom_factor).extend(0f32).extend(0f32)),
        }
    }

    pub fn get_matrix(&self) -> Mat4 {
        self.create_matrix()
    }
}
