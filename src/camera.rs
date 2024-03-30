use glam::{Mat4, Vec2, Vec4, Vec4Swizzles};
use winit::dpi::PhysicalSize;

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
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera_bind_group_layout: wgpu::BindGroupLayout,
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
        camera_buffer: wgpu::Buffer,
        camera_bind_group: wgpu::BindGroup,
        camera_bind_group_layout: wgpu::BindGroupLayout,
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
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
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
        camera_buffer: wgpu::Buffer,
        camera_bind_group: wgpu::BindGroup,
        camera_bind_group_layout: wgpu::BindGroupLayout,
    ) -> Camera {
        let aspect = width / height;
        let left = -aspect / 2.0;
        let right = aspect / 2.0;
        let bottom = -0.5;
        let top = 0.5;
        Camera::new(
            right,
            left,
            top,
            bottom,
            near,
            far,
            zoom_factor,
            position,
            camera_buffer,
            camera_bind_group,
            camera_bind_group_layout,
        )
    }

    pub fn update_matrix_from_screen_size(&mut self, width: f32, height: f32, near: f32, far: f32) {
        let aspect = width / height;
        let left = -aspect / 2.0;
        let right = aspect / 2.0;
        let bottom = -0.5;
        let top = 0.5;

        self.right = right;
        self.left = left;
        self.bottom = bottom;
        self.top = top;
        self.update_matrix();
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

    pub fn update_camera_buffer(&mut self, queue: &wgpu::Queue) {
        queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.uniform]),
        );
    }

    pub fn mouse_to_world(&self, mouse_pos: Vec2, viewport_size: PhysicalSize<u32>) -> Vec2 {
        let viewport_width = viewport_size.width as f32; // Replace with actual viewport width
        let viewport_height = viewport_size.height as f32; // Replace with actual viewport height

        // Normalize mouse coordinates to [-1, 1] range
        let normalized_x = 2.0 * mouse_pos.x / viewport_width - 1.0;
        let normalized_y = -(2.0 * mouse_pos.y / viewport_height) + 1.0; // Invert Y for camera space

        // Inverse viewport transform
        let inv_viewport_width = 1.0 / (self.right - self.left);
        let inv_viewport_height = 1.0 / (self.top - self.bottom);
        let screen_pos = Vec2::new(
            normalized_x * inv_viewport_width * viewport_width,
            normalized_y * inv_viewport_height * viewport_height,
        );

        // Inverse projection transform (assuming orthographic projection)
        let view_to_world = self.create_matrix().inverse(); // Use the camera's inverse matrix
        let world_pos = view_to_world.mul_vec4(screen_pos.extend(0f32).extend(1.0));

        // Return the world position as a Vec2
        world_pos.xy() / self.zoom_factor / 100f32
    }
}
