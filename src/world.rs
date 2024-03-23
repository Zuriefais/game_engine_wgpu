use bytemuck::{Pod, Zeroable};
use glam::{Vec2, Vec3};
use log::info;
use winit::event::WindowEvent;

pub struct WorldObjectContainer {
    pub obj: Box<dyn WorldObject>,
}

pub struct World {
    pub storage: Vec<Box<dyn WorldObject>>,
}

impl World {
    pub fn update(&mut self, delta_t: f32) {
        for object in self.storage.iter_mut() {
            object.update(delta_t)
        }
    }

    pub fn input(&mut self, delta_t: f32, event: &WindowEvent) -> bool {
        for object in self.storage.iter_mut() {
            object.input(delta_t, event);
        }
        true
    }
}

impl Default for World {
    fn default() -> Self {
        Self {
            storage: Default::default(),
        }
    }
}

pub trait WorldObject {
    fn update(&mut self, delta_t: f32) {
        // info!(
        //     "running update for object: {}, delta_t: {}",
        //     self.get_name(),
        //     delta_t
        // );
    }

    fn input(&mut self, delta_t: f32, event: &WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { input, .. } => {
                info!(
                    "input for object: {}, key pressed: {:?}",
                    self.get_name(),
                    input
                );
            }
            WindowEvent::MouseInput { state, button, .. } => {
                info!(
                    "input for object: {}, mouse button pressed: {:?}, state: {:?}",
                    self.get_name(),
                    button,
                    state
                );
            }

            WindowEvent::CursorMoved { position, .. } => {
                info!(
                    "input for object: {}, mouse pos: {:?}",
                    self.get_name(),
                    position
                );
            }
            _ => {}
        }
    }

    fn get_pos(&self) -> Vec2;

    fn render(&self) -> Vec<InstanceData>;

    fn get_name(&self) -> String;
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
pub struct InstanceData {
    pub position: Vec2,
    pub scale: f32,
    pub color: [f32; 4],
}

impl InstanceData {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in the shader.
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}
