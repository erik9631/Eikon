use crate::backend::vulkan::context::{obtain_queues, ContextConfigurator};
use crate::tests::vulkan::log::Logger;
use crate::tests::vulkan::test_utils::{create_test_base, TestApp};
use winit::raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use winit::window::Window;

#[test]
fn context_configurator_init_test() {
    Logger::init(log::LevelFilter::Trace);
    let testfn = |window: &Window| -> ContextConfigurator {
        let base = create_test_base();
        let context_config = ContextConfigurator::new(
            window.window_handle().expect("Failed to get raw window handle").as_raw(),
            window.display_handle().expect("Failed to get raw display handle").as_raw(),
            &["VK_KHR_swapchain"],
        );
        assert!(true);
        context_config
    };
    let mut app = TestApp::new(testfn);
    app.run();
}
#[test]
fn context_configurator_test() {
    Logger::init(log::LevelFilter::Trace);
    let testfn = |window: &Window| -> ContextConfigurator {
        let base = create_test_base();
        let context_config = ContextConfigurator::new(
            window.window_handle().expect("Failed to get raw window handle").as_raw(),
            window.display_handle().expect("Failed to get raw display handle").as_raw(),
            &["VK_KHR_swapchain"],
        );
        let surface = context_config.create_surface(&base);
        let physical_devices = context_config.obtain_physical_devices(&base, &surface);
        assert!(physical_devices.len() > 0);
        let queue_selections = context_config.obtain_queue_families(&base, &physical_devices[0].device, &surface);
        assert!(queue_selections.families.len() > 0);
        let logical_device = context_config.select_logical_device(&base, &queue_selections, &physical_devices[0]);
        let queue_handles = obtain_queues(&logical_device, &queue_selections);
        assert!(queue_handles.queues.len() > 0);
        unsafe { logical_device.destroy_device(None) };
        context_config
    };
    let mut app = TestApp::new(testfn);
    app.run();
}
