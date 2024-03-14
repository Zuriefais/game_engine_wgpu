use log::info;

pub struct World {
    storage: Vec<Box<dyn WorldObject>>
}

impl World {
    pub fn update(&mut self, delta_t: f32) {
        for object in self.storage.iter_mut() {
            object.update(delta_t)
        }
    }
}

pub trait WorldObject {
    fn update(&mut self, delta_t: f32) {
        info!("running update delta_t: {}", delta_t);
    }
}