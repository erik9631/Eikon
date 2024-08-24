use crate::utils::{
    create_logical_device, create_messenger_info, create_surface, create_validation_layers, create_vulcan_instance,
    get_queue_families, pick_physical_device, QueueFamilyIndices,
};
use ash::vk::PhysicalDevice;
use ash::{khr, vk};
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::str::from_utf8_unchecked;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event::WindowEvent::CloseRequested;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::raw_window_handle::HasRawWindowHandle;
use winit::window::{Window, WindowId};

mod utils;

pub struct VulcanApp {
    initialized: bool,
    window: Option<Window>,
    ash_entry: ash::Entry,
    vk_instance: ash::Instance,
    runtime_debugger: ash::ext::debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    selected_physical_device: PhysicalDevice,
    surface_loader: khr::surface::Instance,
    queue_family_indices: Option<QueueFamilyIndices>,
    logical_device: Option<ash::Device>,
    queue: Option<vk::Queue>,
    surface: Option<vk::SurfaceKHR>,
}

impl Drop for VulcanApp {
    fn drop(&mut self) {
        unsafe {
            self.runtime_debugger
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.surface_loader
                .as_ref()
                .unwrap()
                .destroy_surface(self.surface.unwrap(), None);

            self.logical_device.as_ref().unwrap().destroy_device(None);
            self.vk_instance.destroy_instance(None);
        }
    }
}

impl VulcanApp {
    fn create_runtime_debug_ext(
        entry: &ash::Entry,
        instance: &ash::Instance,
    ) -> (ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT) {
        let debug_messenger = create_messenger_info();
        let debug_utils = ash::ext::debug_utils::Instance::new(entry, instance);
        let result = unsafe {
            debug_utils
                .create_debug_utils_messenger(&debug_messenger, None)
                .expect("Failed to create debug utils messenger")
        };
        (debug_utils, result)
    }

    fn verify_validation_layers(
        entry: &ash::Entry,
        mut layers: HashMap<&'static str, (&'static str, bool)>,
    ) -> Vec<*const c_char> {
        let mut vec = Vec::new();
        let properties = unsafe {
            entry
                .enumerate_instance_layer_properties()
                .expect("Failed to enumerate instance layer properties")
        };
        if properties.len() == 0 {
            panic!("No validation layers found");
        }

        for layer in properties {
            let layer_string = unsafe { from_utf8_unchecked(CStr::from_ptr(layer.layer_name.as_ptr()).to_bytes()) };

            if let Some(layer) = layers.remove(layer_string) {
                if layer.1 {
                    vec.push(layer.0.as_ptr() as *const c_char);
                }
            }
        }

        if layers.len() != 0 {
            for layer in layers {
                println!("Layer {} not found", layer.0);
            }
            panic!("Validation layers not found!");
        }
        vec
    }

    pub fn new(validation_layers: HashMap<&'static str, (&'static str, bool)>) -> Self {
        let mut event_loop_builder = EventLoop::builder();
        let event_loop = event_loop_builder
            .with_any_thread(true)
            .build()
            .expect("Failed to create event loop");

        let ash_entry = unsafe { ash::Entry::load().expect("Failed to create entry") };

        let layers = Self::verify_validation_layers(&ash_entry, validation_layers);
        let creation_debugger = create_messenger_info();
        println!("{}", layers.len());

        let instance = create_vulcan_instance(
            &ash_entry,
            layers,
            &creation_debugger as *const vk::DebugUtilsMessengerCreateInfoEXT,
        );
        let (runtime_debugger, debug_utils_messenger) = Self::create_runtime_debug_ext(&ash_entry, &instance);
        let surface_loader = khr::surface::Instance::new(&ash_entry, &instance);
        let physical_device = pick_physical_device(&instance)[0];

        let mut app = Self {
            initialized: false,
            ash_entry,
            vk_instance: instance,
            window: None,
            runtime_debugger,
            debug_utils_messenger,
            surface_loader,
            selected_physical_device: physical_device,
            queue_family_indices: None,
            logical_device: None,
            queue: None,

            surface: None,
        };

        event_loop.run_app(&mut app).expect("Failed to run event loop");
        app
    }
}
impl ApplicationHandler for VulcanApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.initialized {
            return;
        }
        self.initialized = true;

        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window");
        window.set_title("Eikon Engine");
        self.window = Some(window);

        self.surface = Some(create_surface(
            &self.ash_entry,
            &self.vk_instance,
            self.window.as_ref().unwrap(),
        ));
        let surface = self.surface.as_ref().unwrap();

        self.queue_family_indices = Some(get_queue_families(
            &self.vk_instance,
            &self.selected_physical_device,
            &self.surface_loader,
            *surface,
        ));
        let queue_family_indices = self.queue_family_indices.as_ref().unwrap();

        if queue_family_indices.graphics_family.is_none() {
            panic!("No graphics queue family found");
        }

        self.logical_device = Some(create_logical_device(
            &self.vk_instance,
            &self.selected_physical_device,
            &queue_family_indices,
        ));
        let logical_device = self.logical_device.as_ref().unwrap();
        self.queue = Some(unsafe { logical_device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) });
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            _ => {}
        }
    }
}
fn main() {
    let validation_layers = create_validation_layers();
    let app = VulcanApp::new(validation_layers);
}
