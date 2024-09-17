use crate::backend::vulkan::base::Base;
use crate::backend::vulkan::utils::{to_c_str_array, COMPUTE_OP, GRAPHICS_OP, PRESENT_OP, TRANSFER_OP};
use crate::{fatal_assert, fatal_unwrap, fatal_unwrap_e};
use ash::vk::{wl_surface, PhysicalDevice, PhysicalDeviceFeatures, SurfaceCapabilitiesKHR};
use ash::{khr, vk};
use eta_algorithms::algorithms::extract_unique_pairs;
use eta_algorithms::data_structs::array::Array;
use log::{error, info, trace, warn};
use std::collections::HashSet;
use std::ffi::{c_char, c_void, CStr, CString};
use std::mem::transmute;
use std::ptr::null;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub fn default_device_mapper(
    properties: &vk::PhysicalDeviceProperties,
    device_features: &vk::PhysicalDeviceFeatures,
) -> Option<PhysicalDeviceFeatures> {
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return None;
    }
    Some(PhysicalDeviceFeatures::default())
}

pub fn default_queue_mapper(queue_operations: &[u32], queue_family: &[u32]) -> QueueSelections {
    let (operations, families) = extract_unique_pairs(queue_operations, queue_family);
    let mut queue_selections = QueueSelections::new();
    let zipped = operations.iter().zip(families.iter());

    for (operation, family) in zipped {
        match *operation {
            GRAPHICS_OP => {
                queue_selections[QueueSelections::GRAPHICS] = Some(QueueFamily::single(*family));
            }
            COMPUTE_OP => {
                queue_selections[QueueSelections::COMPUTE] = Some(QueueFamily::single(*family));
            }
            TRANSFER_OP => {
                queue_selections[QueueSelections::TRANSFER] = Some(QueueFamily::single(*family));
            }
            PRESENT_OP => {
                queue_selections[QueueSelections::PRESENT] = Some(QueueFamily::single(*family));
            }
            _ => (),
        }
    }
    queue_selections
}

struct SurfaceProperties {
    pub surface_capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}
#[derive(Clone)]
struct QueueFamily {
    pub index: u32,
    pub count: u32,
    pub priorities: Vec<f32>,
}

impl QueueFamily {
    pub fn new(index: u32, count: u32, priorities: Vec<f32>) -> Self {
        Self { index, count, priorities }
    }
    pub fn single(index: u32) -> Self {
        Self::new(index, 1, vec![1.0])
    }
}

struct QueueSelections {
    pub families: Vec<Option<QueueFamily>>,
}

impl QueueSelections {
    const GRAPHICS: usize = 0;
    const COMPUTE: usize = 1;
    const TRANSFER: usize = 2;
    const PRESENT: usize = 3;

    pub fn new() -> Self {
        let mut selections = Self {
            families: Vec::with_capacity(4),
        };
        selections.families.fill(None);
        selections
    }
}
struct PhysicalDeviceInfo {
    device: PhysicalDevice,
    features: PhysicalDeviceFeatures,
    surface_properties: SurfaceProperties,
}

struct ContextConfigurator {
    device_mapper: fn(&vk::PhysicalDeviceProperties, &vk::PhysicalDeviceFeatures) -> Option<PhysicalDeviceFeatures>,
    queue_mapper: fn(&[u32], &[u32]) -> QueueSelections,
    device_extensions: Vec<CString>,
    raw_window_handle: RawWindowHandle,
    raw_display_handle: RawDisplayHandle,
}

impl ContextConfigurator {
    pub fn new(raw_window_handle: RawWindowHandle, raw_display_handle: RawDisplayHandle, device_extensions: &[&str]) -> Self {
        Self {
            device_extensions: to_c_str_array(device_extensions),
            device_mapper: default_device_mapper,
            queue_mapper: default_queue_mapper,
            raw_window_handle,
            raw_display_handle,
        }
    }

    pub fn create_surface(&self, base: &Base) -> (vk::SurfaceKHR, khr::surface::Instance) {
        let surface_instance = khr::surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
        let raw_window_handle = self.raw_window_handle;
        match raw_window_handle {
            /// Win32
            RawWindowHandle::Win32(raw_handle) => {
                let win32_surface_loader = khr::win32_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let surface_info = vk::Win32SurfaceCreateInfoKHR {
                    s_type: Default::default(),
                    p_next: null(),
                    flags: Default::default(),
                    hinstance: raw_handle.hinstance.unwrap().get(),
                    hwnd: raw_handle.hwnd.get(),
                    _marker: Default::default(),
                };

                let platform_surface = unsafe {
                    fatal_unwrap_e!(
                        win32_surface_loader.create_win32_surface(&surface_info, None),
                        "Failed to create surface! {}"
                    )
                };
                (platform_surface, surface_instance)
            }

            /// Linux
            RawWindowHandle::Wayland(raw_handle) => {
                let wayland_surface_loader = khr::wayland_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let display = match self.raw_display_handle {
                    RawDisplayHandle::Wayland(display) => display.display,
                    _ => fatal_assert!("Wayland surfaces must be created with a Wayland display handle!"),
                };

                let surface_info = vk::WaylandSurfaceCreateInfoKHR {
                    s_type: Default::default(),
                    p_next: null(),
                    flags: Default::default(),
                    display: display.as_ptr(),
                    surface: raw_handle.surface.as_ptr(),
                    _marker: Default::default(),
                };
                let platform_surface = unsafe {
                    fatal_unwrap_e!(
                        wayland_surface_loader.create_wayland_surface(&surface_info, None),
                        "Failed to create surface! {}"
                    )
                };
                (platform_surface, surface_instance)
            }
            RawWindowHandle::Xcb(raw_handle) => {
                let xcb_surface_loader = khr::xcb_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let mut display = match self.raw_display_handle {
                    RawDisplayHandle::Xcb(display) => display,
                    _ => fatal_assert!("XCB surfaces must be created with a XCB display handle!"),
                };

                let surface_info = vk::XcbSurfaceCreateInfoKHR {
                    s_type: Default::default(),
                    p_next: null(),
                    flags: Default::default(),
                    connection: fatal_unwrap!(display.connection, "Failed to get XCB connection!").as_ptr(),
                    window: raw_handle.window.get(),
                    _marker: Default::default(),
                };
                let platform_surface = unsafe {
                    fatal_unwrap_e!(
                        xcb_surface_loader.create_xcb_surface(&surface_info, None),
                        "Failed to create surface! {}"
                    )
                };
                (platform_surface, surface_instance)
            }
            RawWindowHandle::Xlib(raw_handle) => {
                let xlib_surface_loader = khr::xlib_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let display = match self.raw_display_handle {
                    RawDisplayHandle::Xlib(display) => display,
                    _ => fatal_assert!("Xlib surfaces must be created with a Xlib display handle!"),
                };

                let surface_info = vk::XlibSurfaceCreateInfoKHR {
                    s_type: Default::default(),
                    p_next: null(),
                    flags: Default::default(),
                    dpy: fatal_unwrap!(display.display, "Failed to get Xlib display!").as_ptr(),
                    window: raw_handle.window,
                    _marker: Default::default(),
                };
                let platform_surface = unsafe {
                    fatal_unwrap_e!(
                        xlib_surface_loader.create_xlib_surface(&surface_info, None),
                        "Failed to create surface! {}"
                    )
                };
                (platform_surface, surface_instance)
            }

            (_) => {
                fatal_assert!("Unsupported window handle type!");
            }
        }
    }

    fn validate_physical_device_extensions(&self, base: &Base, physical_device: PhysicalDevice) -> bool {
        let device_extensions = unsafe {
            fatal_unwrap_e!(
                base.vulkan_instance.enumerate_device_extension_properties(physical_device),
                "Failed to enumerate device extensions {}"
            )
        };
        let mut extension_req: HashSet<CString> = self.device_extensions.iter().map(|extension| extension.clone()).collect();
        for extension in device_extensions.iter() {
            let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };
            if !extension_req.remove(extension_name) {
                trace!("Unsupported device extension found: {:?}", extension_name);
                return false;
            }
        }

        if !extension_req.is_empty() {
            trace!("Missing device extensions!");
            return false;
        }
        true
    }

    fn obtain_device_surface_properties(
        &self,
        surface_instance: &khr::surface::Instance,
        physical_device: &PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> Option<SurfaceProperties> {
        let surface_capabilities = unsafe {
            fatal_unwrap_e!(
                surface_instance.get_physical_device_surface_capabilities(*physical_device, *surface),
                "Failed to get surface capabilities! {}"
            )
        };
        let formats = unsafe {
            fatal_unwrap_e!(
                surface_instance.get_physical_device_surface_formats(*physical_device, *surface),
                "Failed to get surface formats! {}"
            )
        };
        let present_modes = unsafe {
            fatal_unwrap_e!(
                surface_instance.get_physical_device_surface_present_modes(*physical_device, *surface),
                "Failed to get surface present modes! {}"
            )
        };
        if formats.is_empty() || present_modes.is_empty() {
            return None;
        }

        let properties = SurfaceProperties {
            surface_capabilities,
            formats,
            present_modes,
        };
        Some(properties)
    }

    fn obtain_physical_devices(
        &self,
        base: &Base,
        surface_instance: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> Vec<PhysicalDeviceInfo> {
        let checked_devices = unsafe {
            fatal_unwrap_e!(
                base.vulkan_instance.enumerate_physical_devices(),
                "Failed to enumerate physical devices! {}"
            )
        };

        let mut devices = Vec::with_capacity(checked_devices.len());

        for device in checked_devices {
            let properties = unsafe { base.vulkan_instance.get_physical_device_properties(device) };
            let features = unsafe { base.vulkan_instance.get_physical_device_features(device) };
            if !self.validate_physical_device_extensions(&base, device) {
                continue;
            }

            if let Some(surface_properties) = self.obtain_device_surface_properties(surface_instance, &device, surface) {
                if let Some(features) = (self.device_mapper)(&properties, &features) {
                    devices.push(PhysicalDeviceInfo {
                        device,
                        features,
                        surface_properties,
                    });
                    continue;
                }

                trace!("Device does not support required features!");
            }
        }
        devices
    }

    pub fn obtain_queues(
        &self,
        base: &Base,
        surface_instance: &khr::surface::Instance,
        physical_device: &PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> QueueSelections {
        let queue_families = unsafe { base.vulkan_instance.get_physical_device_queue_family_properties(*physical_device) };
        let mut operations = Vec::new();
        let mut families = Vec::new();
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                operations.push(GRAPHICS_OP);
                families.push(index as u32);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                operations.push(COMPUTE_OP);
                families.push(index as u32);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                operations.push(TRANSFER_OP);
                families.push(index as u32);
            }
            let surface_support = unsafe {
                fatal_unwrap_e!(
                    surface_instance.get_physical_device_surface_support(*physical_device, index as u32, *surface),
                    "Failed to get surface support {}"
                )
            };
            if surface_support {
                operations.push(PRESENT_OP);
                families.push(index as u32);
            }
        }

        (self.queue_mapper)(operations.as_slice(), families.as_slice())
    }

    pub fn select_logical_device(
        &self,
        base: &Base,
        queue_selections: &QueueSelections,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> ash::Device {
        let mut queues = Vec::with_capacity(queue_selections.family_count() as usize);
        for queue_family in queue_selections.families.iter() {
            if let Some(queue_family) = queue_family {
                queues.push(vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: null(),
                    flags: Default::default(),
                    queue_family_index: queue_family.index,
                    queue_count: queue_family.count,
                    p_queue_priorities: queue_family.priorities.as_ptr(),
                    _marker: Default::default(),
                });
            }
        }
        let device_extension_list: Vec<*const c_char> = self.device_extensions.iter().map(|extension| extension.as_ptr()).collect();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: null(),
            flags: Default::default(),
            queue_create_info_count: queues.len() as u32,
            p_queue_create_infos: queues.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: null(),
            enabled_extension_count: device_extension_list.len() as u32,
            pp_enabled_extension_names: device_extension_list.as_ptr(),
            p_enabled_features: physical_device_info.features.as_ptr(),
            _marker: Default::default(),
        };

        let device = unsafe {
            base.vulkan_instance
                .create_device(physical_device_info.device, &device_create_info, None)
                .expect("Failed to create device!")
        };
        device
    }
}

struct Context {
    base: Base,
    surface_instance: khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_devices: Vec<PhysicalDeviceInfo>,
    logical_device: ash::Device,
    transfer_queue: vk::Queue,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    compute_queue: vk::Queue,
}

impl Context {
    pub fn new<F, Q>(base: Base, configurator: ContextConfigurator) -> Self {
        let (surface, surface_instance) = configurator.create_surface(&base);
        // TODO fQueues are not needed. Once logical device is created, the queues should be also created internally
        let physical_devices = configurator.obtain_physical_devices(&base, &surface_instance, &surface);
        let queue_selections = configurator.obtain_queues(&base, &surface_instance, &physical_devices[0].device, &surface);
        let logical_device = configurator.select_logical_device(&base, &queue_selections, &physical_devices[0]);
        Self {
            base,
            surface_instance,
            surface,
            logical_device,
            physical_devices,
            transfer_queue: Default::default(),
            graphics_queue: Default::default(),
            present_queue: Default::default(),
            compute_queue: Default::default(),
        }
    }
}
