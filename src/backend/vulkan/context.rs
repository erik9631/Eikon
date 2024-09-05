use crate::backend::vulkan::base::Base;
use crate::backend::vulkan::utils::{COMPUTE_OP, GRAPHICS_OP, PRESENT_OP, TRANSFER_OP};
use crate::{fatal_assert, fatal_unwrap};
use ash::vk::{wl_surface, PhysicalDevice, SurfaceCapabilitiesKHR};
use ash::{khr, vk};
use eta_algorithms::algorithms::extract_unique_pairs;
use log::trace;
use std::collections::HashSet;
use std::ffi::{c_void, CStr, CString};
use std::mem::transmute;
use std::ptr::null;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub fn default_device_mapper(properties: &vk::PhysicalDeviceProperties, features: &vk::PhysicalDeviceFeatures) -> bool {
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return false;
    }
    true
}

pub fn default_queue_mapper(queue_operations: &[u32], queue_family: &[u32]) -> QueueSelections {
    let (operations, families) = extract_unique_pairs(queue_operations, queue_family);
    let mut queue_selections = QueueSelections::new();
    let zipped = operations.iter().zip(families.iter());

    for (operation, family) in zipped {
        match *operation {
            GRAPHICS_OP => queue_selections.graphics = Some(*family),
            COMPUTE_OP => queue_selections.compute = Some(*family),
            TRANSFER_OP => queue_selections.transfer = Some(*family),
            PRESENT_OP => queue_selections.present = Some(*family),
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

struct QueueSelections {
    transfer: Option<u32>,
    graphics: Option<u32>,
    present: Option<u32>,
    compute: Option<u32>,
}

impl QueueSelections {
    pub fn new() -> Self {
        Self {
            transfer: None,
            graphics: None,
            present: None,
            compute: None,
        }
    }
}

struct ContextConfigurator<F, Q>
where
    F: FnMut(&vk::PhysicalDeviceProperties, &vk::PhysicalDeviceFeatures) -> bool,
    Q: FnMut(&[u32], &[u32]) -> QueueSelections,
{
    device_mapper: F,
    queue_mapper: Q,
    device_extensions: Vec<CString>,
    raw_window_handle: RawWindowHandle,
    raw_display_handle: RawDisplayHandle,
}

impl<F, Q> ContextConfigurator<F, Q>
where
    F: FnMut(&vk::PhysicalDeviceProperties, &vk::PhysicalDeviceFeatures) -> bool,
    Q: FnMut(&[u32], &[u32]) -> QueueSelections,
{
    pub fn new(raw_window_handle: RawWindowHandle, raw_display_handle: RawDisplayHandle, device_extensions: &[&str]) -> Self {
        Self {
            device_extensions: device_extensions
                .iter()
                .map(|extension| CString::new(*extension).unwrap())
                .collect(),
            device_mapper: default_device_mapper,
            queue_mapper: default_queue_mapper,
            raw_window_handle,
            raw_display_handle,
        }
    }

    pub fn create_surface<F, Q>(&self, base: &Base) -> (vk::SurfaceKHR, khr::surface::Instance)
    where
        F: FnMut(&vk::PhysicalDeviceProperties, &vk::PhysicalDeviceFeatures) -> bool,
        Q: FnMut(&[u32], &[u32]) -> QueueSelections,
    {
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
                    fatal_unwrap!(
                        win32_surface_loader.create_win32_surface(&surface_info, None),
                        "Failed to create surface!"
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
                    fatal_unwrap!(
                        wayland_surface_loader.create_wayland_surface(&surface_info, None),
                        "Failed to create surface!"
                    )
                };
                (platform_surface, surface_instance)
            }
            RawWindowHandle::Xcb(raw_handle) => {
                let xcb_surface_loader = khr::xcb_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let display = match self.raw_display_handle {
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
                    fatal_unwrap!(
                        xcb_surface_loader.create_xcb_surface(&surface_info, None),
                        "Failed to create surface!"
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
                    fatal_unwrap!(
                        xlib_surface_loader.create_xlib_surface(&surface_info, None),
                        "Failed to create surface!"
                    )
                };
                (platform_surface, surface_instance)
            }

            (_) => {
                fatal_assert!("Unsupported window handle type!");
            }
        }
    }

    fn validate_physical_device_extensions<F, Q>(&self, base: &Base, physical_device: vk::PhysicalDevice) -> bool {
        let device_extensions = unsafe {
            fatal_unwrap!(
                base.vulkan_instance.enumerate_device_extension_properties(physical_device),
                "Failed to enumerate device extensions"
            )
        };
        let mut extension_req: HashSet<CString> = self.device_extensions.iter().map(|extension| *extension).collect();
        for extension in device_extensions.iter() {
            let extension_name = unsafe { CStr::from_ptr(extension.extension_name.as_ptr()) };
            if let None = extension_req.remove(extension_name) {
                trace!("Unsupported device extension found: {:?}", extension);
                return false;
            }
        }

        if !extension_req.is_empty() {
            trace!("Missing device extensions!");
            return false;
        }
        true
    }

    fn validate_surface_support<F, Q>(
        &self,
        surface_instance: &khr::surface::Instance,
        physical_device: &vk::PhysicalDevice,
        surface: &vk::SurfaceKHR,
    ) -> Option<SurfaceProperties> {
        let surface_capabilities = unsafe {
            fatal_unwrap!(
                surface_instance.get_physical_device_surface_capabilities(*physical_device, *surface),
                "Failed to get surface capabilities!"
            )
        };
        let formats = unsafe {
            fatal_unwrap!(
                surface_instance.get_physical_device_surface_formats(*physical_device, *surface),
                "Failed to get surface formats!"
            )
        };
        let present_modes = unsafe {
            fatal_unwrap!(
                surface_instance.get_physical_device_surface_present_modes(*physical_device, *surface),
                "Failed to get surface present modes!"
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

    fn obtain_physical_devices<F, Q>(
        &self,
        base: &Base,
        surface_instance: &khr::surface::Instance,
        surface: &vk::SurfaceKHR,
    ) -> (Vec<vk::PhysicalDevice>, Vec<SurfaceProperties>)
    where
        F: FnMut(&vk::PhysicalDeviceProperties, &vk::PhysicalDeviceFeatures) -> bool,
        Q: FnMut(&[u32], &[u32]) -> QueueSelections,
    {
        let checked_devices = unsafe {
            fatal_unwrap!(
                base.vulkan_instance.enumerate_physical_devices(),
                "Failed to enumerate physical devices!"
            )
        };
        let mut devices = Vec::with_capacity(checked_devices.len());
        let mut surface_properties = Vec::with_capacity(checked_devices.len());

        for device in checked_devices {
            let properties = unsafe { base.vulkan_instance.get_physical_device_properties(device) };
            let features = unsafe { base.vulkan_instance.get_physical_device_features(device) };
            let extensions = unsafe { base.vulkan_instance.enumerate_device_extension_properties(device) };
            if !self.validate_physical_device_extensions(&base, device) {
                continue;
            }

            if let Some(surface_property) = self.validate_surface_support(surface_instance, &device, surface) {
                if !(self.device_mapper)(&properties, &features) {
                    trace!("Device does not support required features!");
                    continue;
                }
                surface_properties.push(surface_property);
                devices.push(device);
            }
        }
        (devices, surface_properties)
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
                fatal_unwrap!(
                    surface_instance.get_physical_device_surface_support(*physical_device, index as u32, *surface),
                    "Failed to get surface support"
                )
            };
            if surface_support {
                operations.push(PRESENT_OP);
                families.push(index as u32);
            }
        }

        (self.queue_mapper)(operations.as_slice(), families.as_slice())
    }
}

struct Context<'a> {
    base: Base,
    surface_instance: khr::surface::Instance,
    surface: vk::SurfaceKHR,
    physical_devices: Vec<vk::PhysicalDevice>,
    physical_device_surface_properties: Vec<SurfaceProperties>,
    transfer_queue: vk::Queue,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    compute_queue: vk::Queue,
    logical_device: vk::Device,
}

impl Context {
    pub fn new<F, Q>(base: Base, configurator: ContextConfigurator<F, Q>) -> Self
    where
        F: FnMut(&vk::PhysicalDeviceProperties, &vk::PhysicalDeviceFeatures) -> bool,
        Q: FnMut(&[u32], &[u32]) -> QueueSelections,
    {
        let (surface, surface_instance) = configurator.create_surface(&base);
        let (physical_devices, physical_device_surface_properties) =
            configurator.obtain_physical_devices(&base, &surface_instance, &surface);
        let queue_selections = configurator.obtain_queues(&base, &surface_instance, &physical_devices[0], &surface);
        Self {
            base,
            surface_instance,
            surface,
            physical_devices,
            physical_device_surface_properties,
            logical_device: Default::default(),
            transfer_queue: Default::default(),
            graphics_queue: Default::default(),
            present_queue: Default::default(),
            compute_queue: Default::default(),
        }
    }
}
