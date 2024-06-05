use crate::enums::cell_assets::CellAssets;
use crate::objects::Player;
use crate::{instance_data::InstanceData, objects::sand::CellWorld};
use glam::Vec2;
use log::info;
use winit::event::WindowEvent;

pub struct WorldObjectContainer {
    pub obj: Box<dyn WorldObject>,
}

#[derive(Default)]
pub struct World {
    pub storage: Vec<Box<dyn WorldObject>>,
    pub assets: CellAssets,
}

impl World {
    pub fn update(&mut self, delta_t: f32) {
        for object in self.storage.iter_mut() {
            object.update(delta_t)
        }
    }

    pub fn input(&mut self, delta_t: f32, event: &WindowEvent, mouse_position: Vec2) -> bool {
        for object in self.storage.iter_mut() {
            object.input(delta_t, event, mouse_position);
        }
        true
    }

    pub fn add_obj(&mut self, obj: Box<dyn WorldObject>) {
        self.storage.push(obj)
    }

    pub fn init_world(assets: CellAssets) -> Self {
        let mut world = World {
            storage: Default::default(),
            assets: assets.clone(),
        };

        let player_obj: Box<dyn WorldObject> = Box::new(Player {
            name: "Main player".to_string(),
            position: Vec2::new(-20.0, -30.0),
        });

        world.add_obj(player_obj);

        let player_obj: Box<dyn WorldObject> = Box::new(Player {
            name: "Main player".to_string(),
            position: Vec2::new(20.0, 50.0),
        });

        world.add_obj(player_obj);

        let sand = Box::new(CellWorld::new(assets.clone()));

        world.add_obj(sand);

        world
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

    fn input(&mut self, delta_t: f32, event: &WindowEvent, mouse_position: Vec2) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                info!(
                    "input for object: {}, key pressed: {:?}",
                    self.get_name(),
                    event
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
