use log::info;
use winit::event::WindowEvent;

use crate::world::WorldObject;

pub struct Player {
    pub(crate) name: String,
}

impl WorldObject for Player {
    fn get_pos(&self) -> glam::Vec2 {
        todo!()
    }

    fn render(&self) -> Vec<crate::world::InstanceData> {
        todo!()
    }

    fn get_name(&self) -> String {
        return self.name.clone();
    }

    fn input(&mut self, delta_t: f32, event: &winit::event::WindowEvent) {
        match event {
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
}
