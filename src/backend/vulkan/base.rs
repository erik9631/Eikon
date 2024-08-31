use crate::backend::vulkan::errors::Error;
use crate::backend::vulkan::errors::Error::ValidationLayerNotSupported;
use crate::backend::vulkan::utils::str_to_version;
use crate::fatal_unwrap;
use ash::vk;
use eta_algorithms::data_structs::array::Array;
use log::{error, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::{null, null_mut};
use winit::raw_window_handle::RawWindowHandle;

pub unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let log_data = CString::from_raw(p_callback_data as *mut c_char).to_str().unwrap();
    let message_type = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[GENERAL]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[PERFORMANCE]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VALIDATION]",
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => "[DEVICE_ADDRESS_BINDING]",
        _ => "UNKNOWN",
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("[ERROR] {} {}", message_type, log_data),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("[WARNING] {}", message_type, log_data),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!("[INFO] {} {}", message_type, log_data),
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => trace!("[VERBOSE] {} {}", message_type, log_data),
        _ => trace!("[UNKNOWN] {} {}", log_data),
    };

    vk::FALSE
}

pub struct BaseConfigurator {
    application_name: CString,
    engine_name: CString,
    validation_layers: Vec<CString>,
    application_version: u32,
    engine_version: u32,
    vulkan_api_version: u32,
    vulkan_extensions: Vec<CString>,
}

impl BaseConfigurator {
    pub fn new(
        application_name: &str,
        engine_name: &str,
        validation_layers: &[&str],
        application_version: &str,
        engine_version: &str,
        vulkan_api_version: &str,
        vulkan_extensions: &[&str],
    ) -> Self {
        Self {
            application_name: CString::new(application_name).unwrap(),
            engine_name: CString::new(engine_name).unwrap(),
            validation_layers: validation_layers.iter().map(|layer| CString::new(*layer).unwrap()).collect(),
            application_version: str_to_version(application_version),
            engine_version: str_to_version(engine_version),
            vulkan_api_version: str_to_version(vulkan_api_version),
            vulkan_extensions: vulkan_extensions
                .iter()
                .map(|extension| CString::new(*extension).unwrap())
                .collect(),
        }
    }

    /// Validates the layers that are requested to be loaded
    /// # Returns
    /// - `Ok(())` if all layers are found
    /// - `Err(layer_name)` if a layer is not found
    pub fn validate_layer_availability(&self, ash_entry: &ash::Entry) -> Result<(), &'static str> {
        let validation_properties = unsafe {
            fatal_unwrap!(
                ash_entry.enumerate_instance_layer_properties(),
                "Failed to enumerate instance layer properties {}"
            )
        };

        let mut layer_list: HashSet<&str> = self.validation_layers.iter().map(|layer| *layer).collect();
        for property in validation_properties.iter() {
            let property_name = unsafe { CStr::from_ptr(property.layer_name.as_ptr()) }.to_str().unwrap();
            if !layer_list.remove(property_name) {
                return Err(property_name);
            }
        }
        if !layer_list.is_empty() {
            return Err(layer_list.iter().next().unwrap());
        }
        Ok(())
    }

    pub fn to_application_info(&self) -> vk::ApplicationInfo {
        vk::ApplicationInfo {
            s_type: Default::default(),
            p_next: null(),
            p_application_name: self.application_name.as_ptr() as *const i8,
            application_version: self.application_version,
            p_engine_name: self.engine_name.as_ptr() as *const i8,
            engine_version: self.engine_version,
            api_version: self.vulkan_api_version,
            _marker: Default::default(),
        }
    }
}

pub struct Base {
    ash_instance: ash::Entry,
    vulkan_instance: ash::Instance,
    utils_instance: ash::ext::debug_utils::Instance,
    debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Base {
    fn new(config: BaseConfigurator) -> Result<Self, Error> {
        let ash_instance = unsafe { fatal_unwrap!(ash::Entry::load(), "Failed to create entry") };
        config
            .validate_layer_availability(&ash_instance)
            .map_err(|layer| ValidationLayerNotSupported(layer))?;

        let application_info = config.to_application_info();
        let validation_layers: Vec<*const c_char> = config.validation_layers.iter().map(|layer| layer.as_ptr()).collect();
        let vulkan_extensions: Vec<*const c_char> = config.vulkan_extensions.iter().map(|layer| layer.as_ptr()).collect();

        let vulkan_create_info = vk::InstanceCreateInfo {
            s_type: Default::default(),
            p_next: null(),
            flags: Default::default(),
            p_application_info: &application_info as *const vk::ApplicationInfo,
            enabled_layer_count: config.validation_layers.len() as u32,
            pp_enabled_layer_names: validation_layers.as_ptr(),
            enabled_extension_count: vulkan_extensions.len() as u32,
            pp_enabled_extension_names: vulkan_extensions.as_ptr(),
            _marker: Default::default(),
        };
        let vulkan_instance = unsafe {
            fatal_unwrap!(
                ash_instance.create_instance(&vulkan_create_info, None),
                "Failed to create instance!"
            )
        };

        let utils_instance = ash::ext::debug_utils::Instance::new(&ash_instance, &vulkan_instance);
        let debug_message_info = vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: Default::default(),
            p_next: null(),
            flags: Default::default(),
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO
                | vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING,
            pfn_user_callback: Some(debug_callback),
            p_user_data: null_mut(),
            _marker: Default::default(),
        };
        let debug_messenger = unsafe {
            fatal_unwrap!(
                utils_instance.create_debug_utils_messenger(&debug_message_info, None),
                "Failed to create debug utils messenger"
            )
        };

        Ok(Self {
            ash_instance,
            utils_instance,
            debug_messenger,
            vulkan_instance,
        })
    }
}

impl Drop for Base {
    fn drop(&mut self) {
        unsafe {
            self.utils_instance.destroy_debug_utils_messenger(self.debug_messenger, None);
            self.vulkan_instance.destroy_instance(None);
        }
    }
}
