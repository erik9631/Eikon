use ash::{khr, vk};

struct Context<'a> {
    vulkan_instance: &'a ash::Instance,
    surface_instance: khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    surface: vk::SurfaceKHR,
    logical_device: vk::Device,
    transfer_queue: vk::Queue,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    compute_queue: vk::Queue,
}
