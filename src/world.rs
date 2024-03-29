use crate::instance_data::InstanceData;
use glam::Vec2;
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

    fn set_pos(&mut self, pos: Vec2);

    fn render(&self) -> Vec<InstanceData>;

    fn get_name(&self) -> String;
}
