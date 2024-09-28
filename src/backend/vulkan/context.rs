use crate::backend::vulkan::base::Base;
use crate::backend::vulkan::queue::op_indices::{queue_flags_to_op_index, COMPUTE, COUNT, GRAPHICS, PRESENT, TRANSFER};
use crate::backend::vulkan::queue::{QueueFamily, QueueHandles, QueueSelections};
use crate::backend::vulkan::surface::Surface;
use crate::backend::vulkan::utils::to_c_str_array;
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
    device_features: &PhysicalDeviceFeatures,
) -> Option<PhysicalDeviceFeatures> {
    if properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU {
        return None;
    }
    Some(PhysicalDeviceFeatures::default())
}

/// OPTIMIZE No need for operations array. We can use bitflags as a hash. No need to do conversions then.
pub fn default_queue_mapper(queue_operations: &[u8], queue_family_indices: &[u32], family_count: u32) -> QueueSelections {
    let (operations, family_index) = extract_unique_pairs(queue_operations, queue_family_indices);
    let mut queue_selections = QueueSelections::new(family_count);
    let zipped = operations.iter().zip(family_index.iter());

    for (operation, family) in zipped {
        fatal_unwrap_e!(
            queue_selections.insert_operation(*operation, *family),
            "Failed to insert operation: {}"
        );
    }
    queue_selections
}

struct SurfaceProperties {
    pub surface_capabilities: SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>,
}

pub struct PhysicalDeviceInfo {
    pub device: PhysicalDevice,
    pub features: PhysicalDeviceFeatures,
    pub surface_properties: SurfaceProperties,
}

pub struct ContextConfigurator {
    device_mapper: fn(&vk::PhysicalDeviceProperties, &PhysicalDeviceFeatures) -> Option<PhysicalDeviceFeatures>,
    queue_mapper: fn(&[u8], &[u32], u32) -> QueueSelections,
    device_extensions: Vec<CString>,
    raw_window_handle: RawWindowHandle,
    raw_display_handle: RawDisplayHandle,
}

impl ContextConfigurator {
    pub fn new(raw_window_handle: RawWindowHandle, raw_display_handle: RawDisplayHandle, device_extensions: &[&str]) -> Self {
        Self {
            device_extensions: to_c_str_array(device_extensions.iter()),
            device_mapper: default_device_mapper,
            queue_mapper: default_queue_mapper,
            raw_window_handle,
            raw_display_handle,
        }
    }

    pub fn create_surface(&self, base: &Base) -> Surface {
        Surface::new(base, self.raw_window_handle, self.raw_display_handle)
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
            extension_req.remove(extension_name);
        }

        if !extension_req.is_empty() {
            for extension in extension_req.iter() {
                trace!("Missing device extension found: {:?}", extension);
            }
            return false;
        }
        true
    }

    fn obtain_device_surface_properties(&self, physical_device: &PhysicalDevice, surface: &Surface) -> Option<SurfaceProperties> {
        let surface_capabilities = surface.get_physical_device_surface_capabilities(&physical_device);
        let formats = surface.get_physical_device_surface_formats(&physical_device);
        let present_modes = surface.get_physical_device_surface_present_modes(&physical_device);
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

    pub fn obtain_physical_devices(&self, base: &Base, surface: &Surface) -> Vec<PhysicalDeviceInfo> {
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
                let name = unsafe { base.vulkan_instance.get_physical_device_properties(device).device_name };
                trace!("Device {:?} does not support required extensions!", unsafe {
                    CStr::from_ptr(name.as_ptr())
                });
                continue;
            }

            if let Some(surface_properties) = self.obtain_device_surface_properties(&device, &surface) {
                if let Some(features) = (self.device_mapper)(&properties, &features) {
                    devices.push(PhysicalDeviceInfo {
                        device,
                        features,
                        surface_properties,
                    });
                    continue;
                }
                let name = unsafe { base.vulkan_instance.get_physical_device_properties(device).device_name };
                trace!("Device {:?} does not support required features!", unsafe {
                    CStr::from_ptr(name.as_ptr())
                });
            }
        }
        devices
    }

    pub fn obtain_queue_families(&self, base: &Base, physical_device: &PhysicalDevice, surface: &Surface) -> QueueSelections {
        let queue_families = unsafe { base.vulkan_instance.get_physical_device_queue_family_properties(*physical_device) };
        let mut operations = Vec::<u8>::new();
        let mut family_indices = Vec::new();
        let family_count = queue_families.len() as u32;
        for (index, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                operations.push(queue_flags_to_op_index(vk::QueueFlags::GRAPHICS) as u8);
                family_indices.push(index as u32);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::COMPUTE) {
                operations.push(queue_flags_to_op_index(vk::QueueFlags::COMPUTE) as u8);
                family_indices.push(index as u32);
            }

            if queue_family.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                operations.push(queue_flags_to_op_index(vk::QueueFlags::TRANSFER) as u8);
                family_indices.push(index as u32);
            }
            let surface_support = surface.get_physical_device_surface_support(*physical_device, index as u32);
            if surface_support {
                operations.push(PRESENT as u8);
                family_indices.push(index as u32);
            }
        }

        (self.queue_mapper)(operations.as_slice(), family_indices.as_slice(), family_count)
    }

    pub fn select_logical_device(
        &self,
        base: &Base,
        queue_selections: &QueueSelections,
        physical_device_info: &PhysicalDeviceInfo,
    ) -> ash::Device {
        let queue_creation_info = queue_selections.to_vk_creation_info();
        let device_extension_list: Vec<*const c_char> = self.device_extensions.iter().map(|extension| extension.as_ptr()).collect();

        let device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: null(),
            flags: Default::default(),
            queue_create_info_count: queue_creation_info.len() as u32,
            p_queue_create_infos: queue_creation_info.as_ptr(),
            enabled_layer_count: 0,
            pp_enabled_layer_names: null(),
            enabled_extension_count: device_extension_list.len() as u32,
            pp_enabled_extension_names: device_extension_list.as_ptr(),
            p_enabled_features: &physical_device_info.features as *const PhysicalDeviceFeatures,
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

pub fn obtain_queues(logical_device: &ash::Device, queue_selections: &QueueSelections) -> QueueHandles {
    let mut obtained_queues = QueueHandles::new();
    for (idx, handle) in queue_selections.operations.iter().enumerate() {
        if let Some(handle) = handle {
            let device_queue = unsafe { logical_device.get_device_queue(handle.index, handle.offset) };
            obtained_queues.queues[idx] = Some(device_queue);
        }
    }
    obtained_queues
}

struct Context {
    base: Base,
    surface: Surface,
    physical_devices: Vec<PhysicalDeviceInfo>,
    logical_device: ash::Device,
    queue_handles: QueueHandles,
}

impl Context {
    pub fn new<F, Q>(base: Base, configurator: ContextConfigurator) -> Self {
        let surface = configurator.create_surface(&base);
        // TODO fQueues are not needed. Once logical device is created, the queues should be also created internally
        let physical_devices = configurator.obtain_physical_devices(&base, &surface);
        let queue_selections = configurator.obtain_queue_families(&base, &physical_devices[0].device, &surface);
        let logical_device = configurator.select_logical_device(&base, &queue_selections, &physical_devices[0]);
        let queue_handles = obtain_queues(&logical_device, &queue_selections);
        Self {
            surface,
            base,
            logical_device,
            physical_devices,
            queue_handles,
        }
    }
}
