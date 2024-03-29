// use glam::*;
// use winit::dpi::PhysicalSize;

// use crate::camera::{self, Camera};

// pub fn ndc_screen_to_world_pos(camera: &Camera, ndc: Vec2) -> Vec2 {
//     let inverse_view_proj = camera.get_matrix().inverse();
//     let world_pos = inverse_view_proj * Vec4::new(ndc.x, ndc.y, 1.0, 1.0).to_homogeneous();
//     world_pos.xy() / world_pos.w
// }

// pub fn world_to_screen_ndc_pos(camera: &Camera, world_pos: Vec2) -> Vec2 {
//     let clip_pos = camera.get_matrix() * Vec4::new(world_pos.x, world_pos.y, 0.0, 1.0).to_homogeneous();
//     let ndc = clip_pos.xy() / clip_pos.w;
//     ndc.clamp(-1.0, 1.0) // Clamp to NDC range
// }

// fn ndc_to_screen(ndc: Vec2, width: f32, height: f32) -> [f32; 2] {
//     // NDC to [0, 1] range
//     let ndc_x = (ndc.x + 1.0) * 0.5;
//     let ndc_y = (ndc.y + 1.0) * 0.5;

//     // [0, 1] range to screen space
//     [ndc_x * width, ndc_y * height]
// }

// fn screen_to_ndc(screen: Vec2, width: f32, height: f32) -> [f32; 2] {
//     // Screen space to [0, 1] range
//     let screen_x = screen.x / width;
//     let screen_y = screen.y / height;

//     // [0, 1] range to NDC
//     [(screen_x * 2.0) - 1.0, (screen_y * 2.0) - 1.0]
// }

pub fn u8_to_f32_safe(bytes: [u8; 4]) -> [f32; 4] {
    let mut floats = [0.0; 4];
    for i in 0..4 {
        floats[i] = f32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]]);
    }
    floats
}
