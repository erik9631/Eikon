use crate::utils::{
    create_command_pool, create_framebuffer, create_image_views, create_logical_device, create_messenger_info, create_pipeline,
    create_surface, create_swap_chain, create_validation_layers_requirements, create_vulcan_instance, get_queue_families,
    get_swapchain_support, pick_physical_device, PipelineInfo, QueueFamilyIndices,
};
use ash::vk::{PhysicalDevice, SurfaceFormatKHR};
use ash::{khr, vk};
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::str::from_utf8_unchecked;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event::WindowEvent::CloseRequested;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::{Window, WindowId};

mod utils;
pub struct Vulkan {
    ash_entry: ash::Entry,
    vk_instance: ash::Instance,
    runtime_debugger: ash::ext::debug_utils::Instance,
    debug_utils_messenger: vk::DebugUtilsMessengerEXT,
    selected_physical_device: PhysicalDevice,
    surface_loader: khr::surface::Instance,
    queue_family_indices: QueueFamilyIndices,
    logical_device: ash::Device,
    queue: vk::Queue,
    surface: vk::SurfaceKHR,
    swap_chain_loader: khr::swapchain::Device,
    swapchain: vk::SwapchainKHR,
    swapchain_size: vk::Extent2D,
    surface_format: SurfaceFormatKHR,
    image_views: Vec<vk::ImageView>,
    pipeline_info: PipelineInfo,
    frame_buffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.runtime_debugger
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.swap_chain_loader.destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.logical_device.destroy_command_pool(self.command_pool, None);

            for frame_buffer in self.frame_buffers.iter() {
                self.logical_device.destroy_framebuffer(*frame_buffer, None);
            }

            self.logical_device.destroy_pipeline(self.pipeline_info.pipeline[0], None);
            self.logical_device
                .destroy_pipeline_layout(self.pipeline_info.pipeline_layout, None);
            self.logical_device.destroy_render_pass(self.pipeline_info.render_pass, None);
            for shader_modules in self.pipeline_info.shaders.values() {
                self.logical_device.destroy_shader_module(*shader_modules, None);
            }

            for image_view in self.image_views.iter() {
                self.logical_device.destroy_image_view(*image_view, None);
            }
            self.logical_device.destroy_device(None);
            self.vk_instance.destroy_instance(None);
        }
    }
}

impl Vulkan {
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

    fn verify_validation_layers(entry: &ash::Entry, mut layers: HashMap<&'static str, (&'static str, bool)>) -> Vec<*const c_char> {
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

    pub fn new(window: &Window) -> Self {
        let validation_layers_requirements = create_validation_layers_requirements();
        let ash_entry = unsafe { ash::Entry::load().expect("Failed to create entry") };
        let layers = Self::verify_validation_layers(&ash_entry, validation_layers_requirements);
        let creation_debugger = create_messenger_info();

        let vk_instance = create_vulcan_instance(
            &ash_entry,
            layers,
            &creation_debugger as *const vk::DebugUtilsMessengerCreateInfoEXT,
        );
        let (runtime_debugger, debug_utils_messenger) = Self::create_runtime_debug_ext(&ash_entry, &vk_instance);
        let surface_loader = khr::surface::Instance::new(&ash_entry, &vk_instance);
        let surface = create_surface(&ash_entry, &vk_instance, window);

        let selected_physical_device = pick_physical_device(&vk_instance, &surface_loader, surface)[0];

        let queue_family_indices = get_queue_families(&vk_instance, &selected_physical_device, &surface_loader, surface);
        let queue_family_indices = queue_family_indices;

        if queue_family_indices.graphics_family.is_none() {
            panic!("No graphics queue family found");
        }

        let logical_device = create_logical_device(&vk_instance, &selected_physical_device, &queue_family_indices);
        let logical_device = logical_device;
        let queue = unsafe { logical_device.get_device_queue(queue_family_indices.graphics_family.unwrap(), 0) };

        let swap_chain_loader = khr::swapchain::Device::new(&vk_instance, &logical_device);
        let swapchain_support = get_swapchain_support(&surface_loader, &selected_physical_device, surface);
        let (swapchain, surface_format, swapchain_size) =
            create_swap_chain(&swap_chain_loader, &swapchain_support, surface, &queue_family_indices, &window);
        let image_views = create_image_views(&logical_device, &swap_chain_loader, &surface_format, &swapchain);
        let pipeline_info = create_pipeline(swapchain_size, &logical_device, &surface_format);
        let frame_buffers = create_framebuffer(&logical_device, swapchain_size, pipeline_info.render_pass, &image_views);
        let command_pool = create_command_pool(&logical_device, &queue_family_indices);

        let mut app = Self {
            ash_entry,
            vk_instance,
            runtime_debugger,
            debug_utils_messenger,
            surface_loader,
            selected_physical_device,
            queue_family_indices,
            logical_device,
            queue,
            surface,
            swap_chain_loader,
            swapchain,
            surface_format,
            image_views,
            swapchain_size,
            pipeline_info,
            frame_buffers,
            command_pool,
        };

        app
    }
}

struct App {
    vulkan: Option<Vulkan>,
    window: Option<Window>,
    event_loop: Option<EventLoop<()>>,
    init: bool,
}

impl App {
    fn new() -> Self {
        let event_loop = Some(EventLoop::<()>::with_user_event().build().expect("Failed to create event loop"));
        Self {
            event_loop,
            vulkan: None,
            window: None,
            init: false,
        }
    }

    fn run(&mut self) {
        let event_loop = self.event_loop.take().unwrap();
        event_loop.run_app(self).expect("Failed to run event loop");
    }
}

impl ApplicationHandler<()> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.init {
            return;
        }
        self.init = true;

        let window = event_loop
            .create_window(Window::default_attributes())
            .expect("Failed to create window");
        window.set_title("Eikon Engine");

        self.vulkan = Some(Vulkan::new(&window));
        self.window = Some(window);
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
    let mut app = App::new();
    app.run();
}
