use crate::backend::vulkan::errors::Error;
use crate::backend::vulkan::errors::Error::ValidationLayerNotSupported;
use crate::backend::vulkan::utils::{to_c_str, to_c_str_array, to_version};
use crate::fatal_unwrap_e;
use crate::{fatal_assert, platform_surface_extension};
use ash::vk;
use ash::Entry;
use ash::Instance;
use ash::{ext, khr};
use eta_algorithms::data_structs::array::Array;
use log::{error, info, trace, warn};
use std::collections::{HashMap, HashSet};
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr::{null, null_mut};

pub unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let log_data = CStr::from_ptr((*p_callback_data).p_message as *mut c_char);
    let message_type = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[GENERAL]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[PERFORMANCE]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VALIDATION]",
        vk::DebugUtilsMessageTypeFlagsEXT::DEVICE_ADDRESS_BINDING => "[DEVICE_ADDRESS_BINDING]",
        _ => "UNKNOWN",
    };

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!("{} {:?}", message_type, log_data),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!("{} {:?}", message_type, log_data),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!("{} {:?}", message_type, log_data),
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => trace!("{} {:?}", message_type, log_data),
        _ => trace!("[{}] {:?}", message_type, log_data),
    };

    vk::FALSE
}

pub fn core_vulkan_extensions() -> Vec<*const c_char> {
    vec![
        khr::surface::NAME.as_ptr(),
        platform_surface_extension!(),
        ext::debug_utils::NAME.as_ptr(),
    ]
}

pub struct BaseConfigBuilder<'a> {
    validation_layers: Option<&'a [&'a str]>,
    vulkan_extensions: Option<&'a [&'a str]>,
}

impl<'a> BaseConfigBuilder<'a> {
    pub fn new() -> Self {
        Self {
            validation_layers: None,
            vulkan_extensions: None,
        }
    }
    pub fn validation_layers(mut self, layers: &'a [&'a str]) -> Self {
        self.validation_layers = Some(layers);
        self
    }
    pub fn use_khronos_validation(mut self) -> Self {
        self.validation_layers = Some(&["VK_LAYER_KHRONOS_validation"]);
        self
    }

    pub fn use_core_vulkan_extensions(mut self) -> Self {
        self
    }
    pub fn vulkan_extensions(mut self, extensions: &'a [&'a str]) -> Self {
        self.vulkan_extensions = Some(extensions);
        self
    }

    pub fn build(
        mut self,
        application_name: &'a str,
        engine_name: &'a str,
        vulkan_api_version: &'a str,
        application_version: &'a str,
        engine_version: &'a str,
    ) -> BaseConfig {
        if self.validation_layers.is_none() {
            self.validation_layers = Some(&[]);
        }

        BaseConfig {
            application_name: to_c_str(application_name),
            engine_name: to_c_str(engine_name),
            validation_layers: to_c_str_array(self.validation_layers.unwrap().iter()),
            application_version: to_version(application_version),
            engine_version: to_version(engine_version),
            vulkan_api_version: to_version(vulkan_api_version),

            // If None then the default is set at initialization. Reason is efficiency.
            // Default is initialized as static CStr. The internal type is CString. We save copy of the default.
            vulkan_extensions: match self.vulkan_extensions {
                Some(extensions) => Some(to_c_str_array(extensions.iter())),
                None => None,
            },
        }
    }
}

pub struct BaseConfig {
    pub application_name: CString,
    pub engine_name: CString,
    pub validation_layers: Vec<CString>,
    pub application_version: u32,
    pub engine_version: u32,
    pub vulkan_api_version: u32,
    pub vulkan_extensions: Option<Vec<CString>>,
}

impl BaseConfig {
    /// Validates the layers that are requested to be loaded
    /// # Returns
    /// - `Ok(())` if all layers are found
    /// - `Err(layer_name)` if a layer is not found
    pub fn validate_layer_availability(&self, ash_entry: &Entry) -> Result<(), Error> {
        let validation_properties = unsafe {
            fatal_unwrap_e!(
                ash_entry.enumerate_instance_layer_properties(),
                "Failed to enumerate instance layer properties {}"
            )
        };

        if validation_properties.len() == 0 {
            return Ok(());
        }

        let mut layer_list: HashSet<&CStr> = self.validation_layers.iter().map(|layer| layer.as_c_str()).collect();

        for requested_layer in validation_properties.iter() {
            let layer_name = unsafe { CStr::from_ptr(requested_layer.layer_name.as_ptr()) };
            layer_list.remove(layer_name);
        }
        if !layer_list.is_empty() {
            let offset = self.validation_layers.len() - layer_list.len();
            return Err(ValidationLayerNotSupported(offset));
        }
        Ok(())
    }

    pub fn to_application_info(&self) -> vk::ApplicationInfo {
        vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
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

// TODO Should work similarly to surface instance. It should be a light wrapper around the vulkan instance.
pub struct Base {
    pub ash_instance: Entry,
    pub vulkan_instance: Instance,
    pub utils_instance: ext::debug_utils::Instance,
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl Base {
    pub fn new(config: BaseConfig) -> Result<Self, Error> {
        let ash_instance = unsafe { fatal_unwrap_e!(Entry::load(), "Failed to create entry {}") };
        config.validate_layer_availability(&ash_instance)?;

        let application_info = config.to_application_info();
        let validation_layers: Vec<*const c_char> = config.validation_layers.iter().map(|layer| layer.as_ptr()).collect();
        let vulkan_extensions: Vec<*const c_char> = match config.vulkan_extensions.as_ref() {
            Some(extensions) => extensions.iter().map(|layer| layer.as_ptr()).collect(),
            None => core_vulkan_extensions(),
        };

        let debug_message_info = vk::DebugUtilsMessengerCreateInfoEXT {
            s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
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

        let vulkan_create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: &debug_message_info as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void,
            flags: Default::default(),
            p_application_info: &application_info as *const vk::ApplicationInfo,
            enabled_layer_count: config.validation_layers.len() as u32,
            pp_enabled_layer_names: validation_layers.as_ptr(),
            enabled_extension_count: vulkan_extensions.len() as u32,
            pp_enabled_extension_names: vulkan_extensions.as_ptr(),
            _marker: Default::default(),
        };
        let vulkan_instance = unsafe {
            fatal_unwrap_e!(
                ash_instance.create_instance(&vulkan_create_info, None),
                "Failed to create instance! {}"
            )
        };

        let utils_instance = ext::debug_utils::Instance::new(&ash_instance, &vulkan_instance);
        let debug_messenger = unsafe {
            fatal_unwrap_e!(
                utils_instance.create_debug_utils_messenger(&debug_message_info, None),
                "Failed to create debug utils messenger {}"
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
