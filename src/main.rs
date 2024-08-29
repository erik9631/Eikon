use crate::utils::{
    create_command_buffers, create_command_pool, create_framebuffer, create_image_views, create_logical_device, create_messenger_info,
    create_pipeline, create_surface, create_swap_chain, create_sync_objects, create_validation_layers_requirements, create_vulcan_instance,
    get_queue_families, get_swapchain_support, pick_physical_device, PipelineInfo, QueueFamilyIndices,
};
use ash::vk::{CommandBuffer, PhysicalDevice, SurfaceFormatKHR};
use ash::{khr, vk};
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::ptr::{null, null_mut};
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
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphore: vk::Semaphore,
    render_finished_semaphore: vk::Semaphore,
    fence: vk::Fence,
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.runtime_debugger
                .destroy_debug_utils_messenger(self.debug_utils_messenger, None);
            self.swap_chain_loader.destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.logical_device.destroy_semaphore(self.image_available_semaphore, None);
            self.logical_device.destroy_semaphore(self.render_finished_semaphore, None);
            self.logical_device.destroy_fence(self.fence, None);
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

    pub fn record_command_buffer(&self, command_buffer: &CommandBuffer, image_index: u32) {
        let clear_values = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.5, 0.0, 0.0, 1.0],
            },
        }];
        let cmd_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: null(),
            flags: vk::CommandBufferUsageFlags::empty(),
            p_inheritance_info: null(),
            _marker: Default::default(),
        };

        unsafe {
            self.logical_device
                .begin_command_buffer(*command_buffer, &cmd_begin_info)
                .expect("Failed to begin command buffer!")
        };

        let render_pass_begin_info = vk::RenderPassBeginInfo {
            s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
            p_next: null(),
            render_pass: self.pipeline_info.render_pass,
            framebuffer: self.frame_buffers[image_index as usize],
            render_area: vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_size,
            },
            clear_value_count: 1,
            p_clear_values: clear_values.as_ptr(),
            _marker: Default::default(),
        };

        unsafe {
            self.logical_device
                .cmd_begin_render_pass(*command_buffer, &render_pass_begin_info, vk::SubpassContents::INLINE)
        };
        unsafe {
            self.logical_device
                .cmd_bind_pipeline(*command_buffer, vk::PipelineBindPoint::GRAPHICS, self.pipeline_info.pipeline[0]);
        }

        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.swapchain_size.width as f32,
            height: self.swapchain_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };

        println!("Viewport: {:?}", viewport);
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: self.swapchain_size,
        };
        println!("Scissor: {:?}", scissor);
        unsafe { self.logical_device.cmd_set_viewport(*command_buffer, 0, &[viewport]) };
        unsafe { self.logical_device.cmd_set_scissor(*command_buffer, 0, &[scissor]) };
        unsafe { self.logical_device.cmd_draw(*command_buffer, 3, 1, 0, 0) };
        unsafe { self.logical_device.cmd_end_render_pass(*command_buffer) };
        unsafe {
            self.logical_device
                .end_command_buffer(*command_buffer)
                .expect("Failed to end command buffer!")
        };
    }

    pub fn draw_frame(&mut self) {
        unsafe {
            self.logical_device
                .wait_for_fences(&[self.fence], true, u64::MAX)
                .expect("Failed to wait for fence!");
        }
        unsafe { self.logical_device.reset_fences(&[self.fence]).expect("Failed to reset fence!") };
        let (current_index, _) = unsafe {
            self.swap_chain_loader
                .acquire_next_image(self.swapchain, u64::MAX, self.image_available_semaphore, vk::Fence::null())
                .expect("Failed to acquire next image!")
        };
        unsafe {
            self.logical_device
                .reset_command_buffer(self.command_buffers[0], vk::CommandBufferResetFlags::RELEASE_RESOURCES)
                .expect("Failed to reset command buffer!");
        };
        self.record_command_buffer(&self.command_buffers[0], current_index);

        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.image_available_semaphore,
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[0],
            signal_semaphore_count: 1,
            p_signal_semaphores: &self.render_finished_semaphore,
            ..Default::default()
        };
        unsafe {
            self.logical_device
                .queue_submit(self.queue, &[submit_info], self.fence)
                .expect("Failed to submit command buffer!")
        };

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: &self.render_finished_semaphore,
            swapchain_count: 1,
            p_swapchains: &self.swapchain as *const vk::SwapchainKHR,
            p_image_indices: &current_index as *const u32,
            p_results: null_mut(),
            _marker: Default::default(),
        };
        unsafe { self.swap_chain_loader.queue_present(self.queue, &present_info) }.expect("TODO: panic message");
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
        let pipeline_info = create_pipeline(&logical_device, &surface_format);
        let frame_buffers = create_framebuffer(&logical_device, swapchain_size, pipeline_info.render_pass, &image_views);
        let command_pool = create_command_pool(&logical_device, &queue_family_indices);
        let command_buffers = create_command_buffers(&logical_device, command_pool);
        let (image_available_semaphore, render_finished_semaphore, fence) = create_sync_objects(&logical_device, &queue_family_indices);

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
            command_buffers,
            image_available_semaphore,
            render_finished_semaphore,
            fence,
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
            WindowEvent::RedrawRequested => {
                self.vulkan.as_mut().unwrap().draw_frame();
            }
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
