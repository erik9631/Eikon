use ash::vk::{CommandBuffer, ComponentMapping, ImageSubresourceRange, PhysicalDevice, SurfaceFormatKHR, SurfaceKHR};
use ash::{ext, khr, vk};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::null;
use std::{fs, ptr};
use winit::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
#[macro_export]
macro_rules! fatal_assert {
    ($($arg:tt)+) => {{
        error!($($arg)+);
        std::process::exit(1);
    }};
}

#[macro_export]
macro_rules! fatal_unwrap_e {
    ($e:expr, $str:expr) => {
        $e.unwrap_or_else(|e| {
            fatal_assert!($str, e);
        })
    };
}
#[macro_export]
macro_rules! fatal_unwrap {
    ($e:expr, $str:expr) => {
        $e.unwrap_or_else(|| {
            fatal_assert!($str);
        })
    };
}

pub struct PipelineInfo {
    pub shaders: HashMap<&'static str, vk::ShaderModule>,
    pub pipeline_layout: vk::PipelineLayout,
    pub render_pass: vk::RenderPass,
    pub pipeline: Vec<vk::Pipeline>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub surface_family: Option<u32>,
    pub priorities: [f32; 1],
}
pub struct SurfaceProperties {
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
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
    extensions.insert(khr::swapchain::NAME.to_str().unwrap(), khr::swapchain::NAME.to_str().unwrap());
    extensions
}
pub fn create_validation_layers_requirements() -> HashMap<&'static str, (&'static str, bool)> {
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
            // return vk::FALSE;
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
            // return vk::FALSE;
            print!("[General]");
        }
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => {
            // return vk::FALSE;
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
    unsafe { entry.create_instance(&create_info, None).expect("Failed to create instance!") }
}

pub fn get_surface_properties(
    surface_loader: &khr::surface::Instance,
    physical_device: &PhysicalDevice,
    surface: SurfaceKHR,
) -> SurfaceProperties {
    let surface_capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(*physical_device, surface)
            .expect("Failed to get surface capabilities")
    };
    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(*physical_device, surface)
            .expect("Failed to get surface formats")
    };
    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(*physical_device, surface)
            .expect("Failed to get surface present modes")
    };
    SurfaceProperties {
        surface_capabilities,
        formats,
        present_modes,
    }
}

pub fn create_surface(
    entry: &ash::Entry,
    instance: &ash::Instance,
    window: &winit::window::Window,
) -> vk::SurfaceKHR {
    let raw_window_handle = window.raw_window_handle().expect("Failed to get raw window handle");
    match raw_window_handle {
        RawWindowHandle::Win32(raw_handle) => {
            let win32_surface_loader = khr::win32_surface::Instance::new(&entry, &instance);

            let surface_info = vk::Win32SurfaceCreateInfoKHR {
                s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
                p_next: null(),
                flags: Default::default(),
                hinstance: raw_handle.hinstance.unwrap().get(),
                hwnd: raw_handle.hwnd.get(),
                _marker: Default::default(),
            };

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
    surface: SurfaceKHR,
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

pub fn check_device_extension_support(
    vulcan_instance: &ash::Instance,
    device: &PhysicalDevice,
) -> bool {
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
    surface: vk::SurfaceKHR,
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

            let surface_properties = get_surface_properties(surface_loader, physical_device, surface);

            if surface_properties.formats.is_empty() | surface_properties.present_modes.is_empty() {
                return false;
            }
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
    let graphics_queue = vk::DeviceQueueCreateInfo {
        s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        queue_family_index: queue_family_indices.surface_family.unwrap(),
        queue_count: 1,
        _marker: Default::default(),
        p_queue_priorities: queue_family_indices.priorities.as_ptr(),
    };

    let surface_queue = vk::DeviceQueueCreateInfo {
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

    let device_queues = [surface_queue, graphics_queue];
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

pub fn select_surface_format(swapchain_support: &SurfaceProperties) -> vk::SurfaceFormatKHR {
    for format in swapchain_support.formats.iter() {
        if format.format == vk::Format::B8G8R8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
            return *format;
        }
    }
    swapchain_support.formats[0]
}

pub fn select_present_mode(swapchain_support: &SurfaceProperties) -> vk::PresentModeKHR {
    for mode in swapchain_support.present_modes.iter() {
        if *mode == vk::PresentModeKHR::FIFO {
            return *mode;
        }
    }
    swapchain_support.present_modes[0]
}

pub fn select_swap_size(
    swapchain_support: &SurfaceProperties,
    window: &winit::window::Window,
) -> vk::Extent2D {
    // If it is already set, the surface must be fixed and pre-configured. We can't change it.
    if swapchain_support.surface_capabilities.current_extent.width != u32::MAX {
        return swapchain_support.surface_capabilities.current_extent;
    }

    // If current_extent is u32::MAX, calculate based on window size
    let window_size = window.inner_size();
    let min_image_extent = swapchain_support.surface_capabilities.min_image_extent;
    let max_image_extent = swapchain_support.surface_capabilities.max_image_extent;

    vk::Extent2D {
        width: window_size.width.clamp(min_image_extent.width, max_image_extent.width),
        height: window_size.height.clamp(min_image_extent.height, max_image_extent.height),
    }
}

pub fn create_swap_chain(
    swap_chain_loader: &khr::swapchain::Device,
    surface_properties: &SurfaceProperties,
    surface: SurfaceKHR,
    queue_family_indices: &QueueFamilyIndices,
    window: &winit::window::Window,
) -> (vk::SwapchainKHR, SurfaceFormatKHR, vk::Extent2D) {
    let surface_format = select_surface_format(surface_properties);
    let present_mode = select_present_mode(surface_properties);
    let extent = select_swap_size(surface_properties, &window);
    let mut image_sharing_mode = vk::SharingMode::EXCLUSIVE;
    let mut queue_family_index_count = 0;
    let mut p_queue_family_indices = null();
    let queue_family_indices_array = [
        queue_family_indices.surface_family.unwrap(),
        queue_family_indices.graphics_family.unwrap(),
    ];
    if queue_family_indices.graphics_family.unwrap() != queue_family_indices.surface_family.unwrap() {
        image_sharing_mode = vk::SharingMode::CONCURRENT;
        p_queue_family_indices = queue_family_indices_array.as_ptr();
        queue_family_index_count = 2;
    }
    let swapchain_create_info = vk::SwapchainCreateInfoKHR {
        s_type: vk::StructureType::SWAPCHAIN_CREATE_INFO_KHR,
        p_next: null(),
        flags: Default::default(),
        surface,
        min_image_count: surface_properties.surface_capabilities.min_image_count + 1,
        image_format: surface_format.format,
        image_color_space: surface_format.color_space,
        image_extent: extent,
        image_array_layers: 1,
        image_usage: vk::ImageUsageFlags::COLOR_ATTACHMENT,
        image_sharing_mode,
        queue_family_index_count,
        p_queue_family_indices,
        pre_transform: surface_properties.surface_capabilities.current_transform,
        composite_alpha: vk::CompositeAlphaFlagsKHR::OPAQUE,
        present_mode,
        clipped: vk::TRUE,
        old_swapchain: vk::SwapchainKHR::null(),
        _marker: Default::default(),
    };
    let swap_chain = unsafe { swap_chain_loader.create_swapchain(&swapchain_create_info, None) };
    (swap_chain.expect("Failed to create Swapchain!"), surface_format, extent)
}

pub fn create_image_views(
    device: &ash::Device,
    swapchain_loader: &khr::swapchain::Device,
    format: &SurfaceFormatKHR,
    swapchain: &vk::SwapchainKHR,
) -> Vec<vk::ImageView> {
    let swapchain_images = unsafe { swapchain_loader.get_swapchain_images(*swapchain) }.expect("Failed to get Swapchain Images.");
    let mut swapchain_image_views = Vec::with_capacity(swapchain_images.len());
    for image in swapchain_images {
        let create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            p_next: null(),
            flags: Default::default(),
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: format.format,
            components: ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
            _marker: Default::default(),
        };
        let image_view = unsafe { device.create_image_view(&create_info, None) }.expect("Failed to create Image View!");
        swapchain_image_views.push(image_view)
    }
    println!("Created {} image views!", swapchain_image_views.len());
    swapchain_image_views
}

pub fn load_shaders(
    logical_device: &ash::Device,
    shader_dir: &str,
) -> HashMap<&'static str, vk::ShaderModule> {
    let fragment_shader = fs::read(shader_dir.to_string() + "/fshader.spv").expect("Failed to read shader file");
    let vertex_shader = fs::read(shader_dir.to_string() + "/vshader.spv").expect("Failed to read shader file");
    let mut byte_shaders = Vec::with_capacity(2);
    byte_shaders.push((fragment_shader, "fshader"));
    byte_shaders.push((vertex_shader, "vshader"));
    let mut shader_modules = HashMap::with_capacity(2);

    for (shader, name) in byte_shaders.iter() {
        if (shader.len() % 4) != 0 {
            panic!("Shader {} is not 4 byte aligned", name);
        }

        let shader = vk::ShaderModuleCreateInfo {
            s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
            p_next: null(),
            flags: Default::default(),
            code_size: shader.len(),
            p_code: shader.as_ptr() as *const u32,
            _marker: Default::default(),
        };
        let shader_module = unsafe { logical_device.create_shader_module(&shader, None) }.expect("Failed to create shader module!");
        println!("Created shader module: {}", name);
        shader_modules.insert(*name, shader_module);
    }
    shader_modules
}
pub fn create_render_pass(
    device: &ash::Device,
    surface_format: &SurfaceFormatKHR,
) -> vk::RenderPass {
    let color_attachment = vk::AttachmentDescription {
        flags: Default::default(),
        format: surface_format.format,
        samples: vk::SampleCountFlags::TYPE_1,
        load_op: vk::AttachmentLoadOp::CLEAR,
        store_op: vk::AttachmentStoreOp::STORE,
        stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        initial_layout: vk::ImageLayout::UNDEFINED,
        final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
    };

    let color_attachment_ref = vk::AttachmentReference {
        attachment: 0,
        layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
    };

    let subpass = vk::SubpassDescription {
        flags: Default::default(),
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        input_attachment_count: 0,
        p_input_attachments: null(),
        color_attachment_count: 1,
        p_color_attachments: &color_attachment_ref,
        p_resolve_attachments: null(),
        p_depth_stencil_attachment: null(),
        preserve_attachment_count: 0,
        p_preserve_attachments: null(),
        _marker: Default::default(),
    };

    let subpass_dependency = vk::SubpassDependency {
        src_subpass: vk::SUBPASS_EXTERNAL,
        dst_subpass: 0,
        src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        dependency_flags: vk::DependencyFlags::BY_REGION,
    };

    let render_pass = vk::RenderPassCreateInfo {
        s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        attachment_count: 1,
        p_attachments: &color_attachment as *const vk::AttachmentDescription,
        subpass_count: 1,
        p_subpasses: &subpass as *const vk::SubpassDescription,
        dependency_count: 1,
        p_dependencies: &subpass_dependency as *const vk::SubpassDependency,
        _marker: Default::default(),
    };

    unsafe { device.create_render_pass(&render_pass, None) }.expect("Failed to create render pass!")
}

pub fn create_pipeline(
    logical_device: &ash::Device,
    format: &SurfaceFormatKHR,
) -> PipelineInfo {
    let shaders = load_shaders(&logical_device, "cshaders");
    let render_pass = create_render_pass(&logical_device, format);

    let main_function_name = CString::new("main").unwrap(); // the beginning function name in shader code.

    let vertex_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        module: shaders["vshader"],
        p_name: main_function_name.as_ptr(),
        p_specialization_info: null(),
        stage: vk::ShaderStageFlags::VERTEX,
        _marker: Default::default(),
    };

    let fragment_shader_stage_info = vk::PipelineShaderStageCreateInfo {
        s_type: vk::StructureType::PIPELINE_SHADER_STAGE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        module: shaders["fshader"],
        p_name: main_function_name.as_ptr(),
        p_specialization_info: null(),
        stage: vk::ShaderStageFlags::FRAGMENT,
        _marker: Default::default(),
    };
    let stages = [vertex_shader_stage_info, fragment_shader_stage_info];

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let pipeline_vertex_input_state = vk::PipelineVertexInputStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        vertex_binding_description_count: 0,
        p_vertex_binding_descriptions: null(),
        vertex_attribute_description_count: 0,
        p_vertex_attribute_descriptions: null(),
        _marker: Default::default(),
    };

    let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        topology: vk::PrimitiveTopology::TRIANGLE_LIST,
        primitive_restart_enable: vk::FALSE,
        _marker: Default::default(),
    };

    let viewport_state = vk::PipelineViewportStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_VIEWPORT_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        viewport_count: 1,
        p_viewports: null(),
        scissor_count: 1,
        p_scissors: null(),
        _marker: Default::default(),
    };

    let rasterizer = vk::PipelineRasterizationStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        depth_clamp_enable: vk::FALSE,
        rasterizer_discard_enable: vk::FALSE,
        polygon_mode: vk::PolygonMode::FILL,
        cull_mode: vk::CullModeFlags::NONE,
        front_face: vk::FrontFace::CLOCKWISE,
        depth_bias_enable: 0,
        depth_bias_constant_factor: 0.0,
        depth_bias_clamp: 0.0,
        depth_bias_slope_factor: 0.0,
        line_width: 1.0,
        _marker: Default::default(),
    };

    let multisample_state = vk::PipelineMultisampleStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        rasterization_samples: vk::SampleCountFlags::TYPE_1,
        sample_shading_enable: vk::FALSE,
        min_sample_shading: 1.0,
        p_sample_mask: null(),
        alpha_to_coverage_enable: vk::FALSE,
        alpha_to_one_enable: vk::FALSE,
        _marker: Default::default(),
    };

    let color_blending_attachment = vk::PipelineColorBlendAttachmentState {
        blend_enable: vk::TRUE,
        src_color_blend_factor: vk::BlendFactor::SRC_ALPHA,
        dst_color_blend_factor: vk::BlendFactor::ONE_MINUS_SRC_ALPHA,
        color_blend_op: vk::BlendOp::ADD,
        src_alpha_blend_factor: vk::BlendFactor::ONE,
        dst_alpha_blend_factor: vk::BlendFactor::ZERO,
        alpha_blend_op: Default::default(), // vk::BlendOp::ADD,
        color_write_mask: vk::ColorComponentFlags::R | vk::ColorComponentFlags::G | vk::ColorComponentFlags::B | vk::ColorComponentFlags::A,
    };

    let color_blending_state = vk::PipelineColorBlendStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        logic_op_enable: vk::FALSE,
        logic_op: vk::LogicOp::COPY,
        attachment_count: 1,
        p_attachments: &color_blending_attachment as *const vk::PipelineColorBlendAttachmentState,
        blend_constants: [0.0, 0.0, 0.0, 0.0],
        _marker: Default::default(),
    };

    let pipline_dynamic_state = vk::PipelineDynamicStateCreateInfo {
        s_type: vk::StructureType::PIPELINE_DYNAMIC_STATE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        dynamic_state_count: 2,
        p_dynamic_states: dynamic_states.as_ptr(),
        _marker: Default::default(),
    };

    let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo {
        s_type: vk::StructureType::PIPELINE_LAYOUT_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        set_layout_count: 0,
        p_set_layouts: null(),
        push_constant_range_count: 0,
        p_push_constant_ranges: null(),
        _marker: Default::default(),
    };

    let pipeline_layout = unsafe {
        logical_device
            .create_pipeline_layout(&pipeline_layout_create_info, None)
            .expect("Failed to create pipeline layout!")
    };

    let pipline_info = vk::GraphicsPipelineCreateInfo {
        s_type: vk::StructureType::GRAPHICS_PIPELINE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        stage_count: stages.len() as u32,
        p_stages: stages.as_ptr(),
        p_vertex_input_state: &pipeline_vertex_input_state as *const vk::PipelineVertexInputStateCreateInfo,
        p_input_assembly_state: &input_assembly_state as *const vk::PipelineInputAssemblyStateCreateInfo,
        p_tessellation_state: null(),
        p_viewport_state: &viewport_state as *const vk::PipelineViewportStateCreateInfo,
        p_rasterization_state: &rasterizer as *const vk::PipelineRasterizationStateCreateInfo,
        p_multisample_state: &multisample_state as *const vk::PipelineMultisampleStateCreateInfo,
        p_depth_stencil_state: null(),
        p_color_blend_state: &color_blending_state as *const vk::PipelineColorBlendStateCreateInfo,
        p_dynamic_state: &pipline_dynamic_state as *const vk::PipelineDynamicStateCreateInfo,
        layout: pipeline_layout,
        render_pass,
        subpass: 0,
        base_pipeline_handle: vk::Pipeline::null(),
        base_pipeline_index: 0,
        _marker: Default::default(),
    };
    let pipeline = unsafe {
        logical_device
            .create_graphics_pipelines(vk::PipelineCache::null(), &[pipline_info], None)
            .expect("Failed to create pipeline!")
    };
    PipelineInfo {
        shaders,
        pipeline_layout,
        render_pass,
        pipeline,
    }
}

pub fn create_framebuffer(
    device: &ash::Device,
    swapchain_size: vk::Extent2D,
    render_pass: vk::RenderPass,
    image_views: &Vec<vk::ImageView>,
) -> Vec<vk::Framebuffer> {
    let mut vec = Vec::with_capacity(image_views.len());
    for view in image_views.iter() {
        let framebuffer_create_info = vk::FramebufferCreateInfo {
            s_type: vk::StructureType::FRAMEBUFFER_CREATE_INFO,
            p_next: null(),
            flags: Default::default(),
            render_pass,
            attachment_count: 1,
            p_attachments: view as *const vk::ImageView,
            width: swapchain_size.width,
            height: swapchain_size.height,
            layers: 1,
            _marker: Default::default(),
        };
        let framebuffer = unsafe { device.create_framebuffer(&framebuffer_create_info, None) }.expect("Failed to create framebuffer!");
        vec.push(framebuffer);
    }
    vec
}

pub fn create_command_pool(
    device: &ash::Device,
    queue_family_indices: &QueueFamilyIndices,
) -> vk::CommandPool {
    let command_pool_create_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: null(),
        flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
        queue_family_index: queue_family_indices.graphics_family.unwrap(),
        _marker: Default::default(),
    };
    unsafe { device.create_command_pool(&command_pool_create_info, None) }.expect("Failed to create command pool!")
}

pub fn create_command_buffers(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> Vec<CommandBuffer> {
    let vk_command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: null(),
        command_buffer_count: 1,
        command_pool,
        level: vk::CommandBufferLevel::PRIMARY,
        _marker: Default::default(),
    };
    let command_buffers =
        unsafe { device.allocate_command_buffers(&vk_command_buffer_allocate_info) }.expect("Failed to allocate command buffers!");
    command_buffers
}

pub fn create_sync_objects(
    device: &ash::Device,
    queue_family_indices: &QueueFamilyIndices,
) -> (vk::Semaphore, vk::Semaphore, vk::Fence) {
    let semaphore_create_info = vk::SemaphoreCreateInfo {
        s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
        p_next: null(),
        flags: Default::default(),
        _marker: Default::default(),
    };
    let image_available_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None) }.expect("Failed to create semaphore!");
    let render_finished_semaphore = unsafe { device.create_semaphore(&semaphore_create_info, None) }.expect("Failed to create semaphore!");

    let fence_create_info = vk::FenceCreateInfo {
        s_type: vk::StructureType::FENCE_CREATE_INFO,
        p_next: null(),
        flags: vk::FenceCreateFlags::SIGNALED,
        _marker: Default::default(),
    };
    let fence = unsafe { device.create_fence(&fence_create_info, None) }.expect("Failed to create fence!");
    (image_available_semaphore, render_finished_semaphore, fence)
}
