pub mod app;
pub mod app_state;
pub mod camera;
pub mod constants;
pub mod enums;
pub mod instance_data;
pub mod objects;
pub mod utils;
pub mod world;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::default();
    if let Err(e) = event_loop.run_app(&mut app) {
        eprintln!("Event loop error: {e}");
    }
}
