pub mod sand;

use glam::Vec2;
use winit::{
    event::WindowEvent,
    keyboard::{KeyCode, PhysicalKey},
};

use crate::{instance_data::InstanceData, world::WorldObject};

pub struct Player {
    pub(crate) name: String,
    pub(crate) position: Vec2,
}

impl WorldObject for Player {
    fn get_pos(&self) -> Vec2 {
        self.position
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }

    fn render(&self) -> Vec<crate::instance_data::InstanceData> {
        let mut instances = vec![];
        for x in 15..20 {
            for y in 15..20 {
                instances.push(InstanceData {
                    position: Vec2::new(x as f32, y as f32) + self.position,
                    color: 0,
                })
            }
        }
        instances
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn update(&mut self, delta_t: f32) {
        // info!("{}", self.get_pos().to_string())
    }

    fn input(&mut self, delta_t: f32, event: &winit::event::WindowEvent, mouse_position: Vec2) {
        match event {
            WindowEvent::CursorMoved { .. } => {
                // info!(
                //     "input for object: {}, mouse pos: {:?}",
                //     self.get_name(),
                //     position
                // );
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let mut direction = Vec2::ZERO;
                match event.physical_key {
                    PhysicalKey::Code(KeyCode::KeyW) => {
                        direction.y = 1.0;
                    }
                    PhysicalKey::Code(KeyCode::KeyA) => {
                        direction.x = -1.0;
                    }
                    PhysicalKey::Code(KeyCode::KeyS) => {
                        direction.y = -1.0;
                    }
                    PhysicalKey::Code(KeyCode::KeyD) => {
                        direction.x = 1.0;
                    }
                    _ => {}
                }
                if direction != Vec2::ZERO {
                    self.position += direction.normalize() * 0.9;
                }
            }
            _ => {}
        }
    }
}
