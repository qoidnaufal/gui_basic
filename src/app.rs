use winit::{
    application::ApplicationHandler,
    event::{self, WindowEvent},
    event_loop::ActiveEventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

use crate::video_player::VideoPlayer;

#[derive(Default)]
pub struct App {
    pub window: Option<Window>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let config = Window::default_attributes()
            .with_maximized(false)
            .with_title("My Basic GUI");

        let window = event_loop.create_window(config).unwrap();

        self.window = Some(window);

        let instance = wgpu::Instance::new(Default::default());
        let surface = instance
            .create_surface(self.window.as_ref().unwrap())
            .expect("Unable to create surface from the window");
        futures::executor::block_on(async move {
            let adapter = instance
                .request_adapter(&wgpu::RequestAdapterOptionsBase {
                    power_preference: wgpu::PowerPreference::default(),
                    force_fallback_adapter: false,
                    compatible_surface: Some(&surface),
                })
                .await
                .expect("Unable to request adapter from the surface");
            println!("{:?}", adapter);
        });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("Close button was pressed, stopping...");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowEvent::KeyboardInput {
                event:
                    event::KeyEvent {
                        state: event::ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::KeyO),
                        ..
                    },
                ..
            } => {
                // another option to spawn std::thread
                // and use futures::executor to block on and run the async code

                // std::thread::Builder::new()
                //     .name("open file".into())
                //     .spawn(move || {
                //         futures::executor::block_on(async move {});
                //     })
                //     .unwrap()
                //     .join()
                //     .unwrap();

                tokio::spawn(async move {
                    let video_player = VideoPlayer::open_file().await;
                    video_player.decode_file();
                });
            }
            _ => (), // n => println!("{:?}", n),
        }
    }
}
