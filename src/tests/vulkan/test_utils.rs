use crate::backend::vulkan::base::{Base, BaseConfigBuilder};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event::WindowEvent::CloseRequested;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::{Window, WindowId};

pub struct TestApp<T> {
    init_function: fn(&Window) -> T,
    window: Option<Window>,
    render_instance: Option<T>,
    init: bool,
}

impl<T> TestApp<T> {
    pub fn new(init_function: fn(window: &Window) -> T) -> Self {
        Self {
            init_function,
            window: None,
            render_instance: None,
            init: false,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::<()>::builder()
            .with_any_thread(true)
            .build()
            .expect("Failed to create event loop");
        event_loop.run_app(self).expect("Failed to run event loop");
    }
}

impl<T> ApplicationHandler<()> for TestApp<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.init {
            return;
        }
        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window");
        self.window = Some(window);
        self.render_instance = Some((self.init_function)(self.window.as_ref().unwrap()));
        self.init = true;
        event_loop.exit();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            CloseRequested => {
                event_loop.exit();
            }
            _ => {
                if self.init {
                    event_loop.exit();
                }
            }
        }
    }
}

pub fn create_test_base() -> Base {
    let base_builder = BaseConfigBuilder::new();
    let base_config = base_builder
        .use_khronos_validation()
        .use_core_vulkan_extensions()
        .build("Test", "Test", "1.0.0", "1.0.0", "1.0.0");
    Base::new(base_config).expect("Failed to create base!")
}
