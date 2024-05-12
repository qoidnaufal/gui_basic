use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
};

use gui_basic::app::App;

#[tokio::main]
async fn main() -> Result<(), EventLoopError> {
    env_logger::init();
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::default();

    event_loop.run_app(&mut app)
}
