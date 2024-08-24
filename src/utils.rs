use ash::vk::PhysicalDevice;
use ash::{ext, khr, vk};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr};
use std::ptr;
use std::ptr::null;
use winit::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub surface_family: Option<u32>,
    pub priorities: [f32; 1],
}
pub struct SwapChainSupportDetails {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
impl QueueFamilyIndices {
    fn new(family_index: Option<u32>) -> Self {
        Self {
            graphics_family: family_index,
            surface_family: family_index,
            priorities: [1.0],
        }
    }
}

pub const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
pub const ENGINE_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
pub const API_VERSION: u32 = vk::API_VERSION_1_3;
pub const REQUIRED_EXTENSION_NAMES: &[*const c_char] = &[
    khr::surface::NAME.as_ptr(),
    khr::win32_surface::NAME.as_ptr(),
    ext::debug_utils::NAME.as_ptr(),
];

pub const REQUIRED_DEVICE_EXTENSIONS: &[*const c_char] = &[khr::swapchain::NAME.as_ptr()];

pub fn create_physical_device_extension_requirements() -> HashMap<&'static str, &'static str> {
    let mut extensions = HashMap::new();
    extensions.insert(
        khr::swapchain::NAME.to_str().unwrap(),
        khr::swapchain::NAME.to_str().unwrap(),
    );
    extensions
}
pub fn create_validation_layers() -> HashMap<&'static str, (&'static str, bool)> {
    let mut layers = HashMap::new();
    layers.insert("VK_LAYER_KHRONOS_validation", ("VK_LAYER_KHRONOS_validation\0", true));
    layers
}

pub unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            print!("[Error]");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            print!("[Warning]");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => {
            print!("[Info]");
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
            print!("[Verbose]");
        }
        _ => {
            print!("[Unknown]");
        }
    }

    match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => {
            print!("[General]");
        }
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => {
            print!("[Performance]");
        }
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => {
            print!("[Validation]");
        }
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => {
            print!("[Device address binding]");
        }
        _ => {
            print!("[Unknown] ");
        }
    }
    let str = CStr::from_ptr((*p_callback_data).p_message);
    println!("{:?}", str);
    vk::FALSE
}

pub fn create_messenger_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
            | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
            | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
            | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
        p_next: ptr::null(),
        pfn_user_callback: Some(debug_callback),
        p_user_data: ptr::null_mut(),
        _marker: Default::default(),
    }
}

pub fn create_vulcan_instance(
    entry: &ash::Entry,
    validation_layers: Vec<*const c_char>,
    debug_struct_info: *const vk::DebugUtilsMessengerCreateInfoEXT,
) -> ash::Instance {
    let application_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: "Vulkan Engine".as_ptr() as *const i8,
        application_version: APPLICATION_VERSION,
        p_engine_name: "Vulkan Engine".as_ptr() as *const i8,
        engine_version: ENGINE_VERSION,
        api_version: API_VERSION,
        _marker: Default::default(),
    };

    let create_info = vk::InstanceCreateInfo {
        s_type: vk::StructureType::INSTANCE_CREATE_INFO,
        p_next: debug_struct_info as *const c_void,
        flags: vk::InstanceCreateFlags::empty(),
        p_application_info: &application_info,
        pp_enabled_layer_names: validation_layers.as_ptr(),
        enabled_layer_count: validation_layers.len() as u32,
        pp_enabled_extension_names: REQUIRED_EXTENSION_NAMES.as_ptr(),
        enabled_extension_count: REQUIRED_EXTENSION_NAMES.len() as u32,
        _marker: Default::default(),
    };
    unsafe {
        entry
            .create_instance(&create_info, None)
            .expect("Failed to create instance!")
    }
}

pub fn qet_swapchain_support(
    surface_loader: &khr::surface::Instance,
    physical_device: &PhysicalDevice,
    surface: &vk::SurfaceKHR,
) -> SwapChainSupportDetails {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*physical_device, *surface)
            .expect("Failed to get surface capabilities")
    };
    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*physical_device, *surface)
            .expect("Failed to get surface formats")
    };
    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*physical_device, *surface)
            .expect("Failed to get surface present modes")
    };
    SwapChainSupportDetails {
        capabilities,
        formats,
        present_modes,
    }
}

pub fn create_surface(entry: &ash::Entry, instance: &ash::Instance, window: &winit::window::Window) -> vk::SurfaceKHR {
    let raw_window_handle = window.raw_window_handle().expect("Failed to get raw window handle");
    match raw_window_handle {
        RawWindowHandle::Win32(raw_handle) => {
            let surface_info = vk::Win32SurfaceCreateInfoKHR {
                s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
                p_next: null(),
                flags: Default::default(),
                hinstance: raw_handle.hinstance.unwrap().get(),
                hwnd: raw_handle.hwnd.get(),
                _marker: Default::default(),
            };
            let win32_surface_loader = khr::win32_surface::Instance::new(entry, instance);
            let platform_surface = unsafe {
                win32_surface_loader
                    .create_win32_surface(&surface_info, None)
                    .expect("Failed to create surface!")
            };
            platform_surface
        }
        _ => panic!("Unsupported window handle type"),
    }
}

pub fn get_queue_families(
    vulcan_instance: &ash::Instance,
    device: &PhysicalDevice,
    surface_loader: &khr::surface::Instance,
    surface: vk::SurfaceKHR,
) -> QueueFamilyIndices {
    let queue_families = unsafe { vulcan_instance.get_physical_device_queue_family_properties(*device) };
    let mut graphics_family = QueueFamilyIndices::new(None);

    for (index, queue_family) in queue_families.iter().enumerate() {
        if graphics_family.graphics_family.is_some() && graphics_family.surface_family.is_some() {
            break;
        }

        if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
            if graphics_family.graphics_family.is_none() {
                println!("Set graphics queue family idx: {}", index);
                graphics_family.graphics_family = Some(index as u32);
            }
        }

        if unsafe { surface_loader.get_physical_device_surface_support(*device, index as u32, surface) }
            .expect("Failed to get surface support")
        {
            if graphics_family.surface_family.is_none() {
                println!("Set surface queue family idx: {}", index);
                graphics_family.surface_family = Some(index as u32);
            }
            break;
        }
    }
    if graphics_family.graphics_family.is_none() {
        panic!("No graphics queue found!");
    }
    if graphics_family.surface_family.is_none() {
        panic!("No surface queue found!");
    }
    graphics_family
}

pub fn check_device_extension_support(vulcan_instance: &ash::Instance, device: &PhysicalDevice) -> bool {
    let extensions = unsafe {
        vulcan_instance
            .enumerate_device_extension_properties(*device)
            .expect("Failed to enumerate device extensions")
    };
    let mut requested_extensions = create_physical_device_extension_requirements();

    for extension in extensions.iter() {
        let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()).to_str().unwrap() };
        let result = requested_extensions.remove(extension_name);
        if result.is_some() {
            println!("Supported device extension found: {}", extension_name);
        }
    }

    if requested_extensions.len() > 0 {
        return false;
    }
    println!("All requested device extensions supported!");
    true
}
pub fn pick_physical_device(
    vulcan_instance: &ash::Instance,
    surface_loader: &khr::surface::Instance,
) -> Vec<PhysicalDevice> {
    let physical_devices = unsafe {
        vulcan_instance
            .enumerate_physical_devices()
            .expect("Failed to enumerate physical devices")
    };
    if physical_devices.len() == 0 {
        panic!("No physical devices supporting Vulkan found!");
    }

    let filtered_physical_devices: Vec<PhysicalDevice> = physical_devices
        .iter()
        .cloned()
        .filter(|physical_device| {
            let properties = unsafe { vulcan_instance.get_physical_device_properties(*physical_device) };
            if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
                return false;
            }

            let features = unsafe { vulcan_instance.get_physical_device_features(*physical_device) };
            if !features.geometry_shader == vk::FALSE {
                return false;
            }
            if !check_device_extension_support(vulcan_instance, physical_device) {
                return false;
            }
            let swapchain_support = qet_swapchain_support(surface_loader, physical_device, surface);

            return true;
        })
        .collect();

    for physical_device in filtered_physical_devices.iter() {
        let properties = unsafe { vulcan_instance.get_physical_device_properties(*physical_device) };
        println!("Name: {:?}", unsafe { CStr::from_ptr(properties.device_name.as_ptr()) });
        println!("===============")
    }
    filtered_physical_devices
}

pub fn create_logical_device(
    vulcan_instance: &ash::Instance,
    physical_device: &PhysicalDevice,
    queue_family_indices: &QueueFamilyIndices,
) -> ash::Device {
    let surface_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        queue_family_index: queue_family_indices.surface_family.unwrap(),
        queue_count: 1,
        _marker: Default::default(),
        p_queue_priorities: queue_family_indices.priorities.as_ptr(),
    };

    let graphics_info = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        queue_family_index: queue_family_indices.graphics_family.unwrap(),
        queue_count: 1,
        _marker: Default::default(),
        p_queue_priorities: queue_family_indices.priorities.as_ptr(),
    };

    let mut count = 1;
    if queue_family_indices.surface_family.unwrap() != queue_family_indices.graphics_family.unwrap() {
        count = 2;
    }

    let device_queues = [graphics_info, surface_info];
    let physical_device_features = vk::PhysicalDeviceFeatures { ..Default::default() };

    let create_device_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        queue_create_info_count: count as u32,
        p_queue_create_infos: device_queues.as_ptr(),
        enabled_layer_count: 0,
        pp_enabled_layer_names: null(),
        enabled_extension_count: 1,
        pp_enabled_extension_names: REQUIRED_DEVICE_EXTENSIONS.as_ptr(),
        p_enabled_features: &physical_device_features,
        _marker: Default::default(),
    };
    unsafe {
        vulcan_instance
            .create_device(*physical_device, &create_device_info, None)
            .expect("Failed to create device")
    }
}

pub fn init_swap_chain() {}

pub fn init_commands() {}

pub fn init_sync() {}
