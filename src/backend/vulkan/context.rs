use ash::{khr, vk};

struct ContextConfigurator<F, Q>
where
    F: FnMut(vk::PhysicalDevice) -> bool,
    Q: FnMut(vk::QueueFamilyProperties) -> vk::QueueFlags,
{
    minimum_device_requirements: vk::PhysicalDevice,
    device_mapper: F,
    queue_mapper: Q,
}

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

impl Context {
    pub fn new(config: ContextConfigurator) -> Self {
        Self {
            vulkan_instance: &(),
            surface_instance: (),
            physical_device: Default::default(),
            surface: Default::default(),
            logical_device: Default::default(),
            transfer_queue: Default::default(),
            graphics_queue: Default::default(),
            present_queue: Default::default(),
            compute_queue: Default::default(),
        }
    }
}
