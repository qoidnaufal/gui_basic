use std::error::Error;

use winit::event_loop::{ControlFlow, EventLoop};

use gui_basic::App;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    app.set_bg_color(&[0.824, 0.902, 0.698, 1.0]);

    event_loop.run_app(&mut app).map_err(|err| err.into())
}
