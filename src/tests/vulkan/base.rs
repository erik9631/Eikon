use crate::backend::vulkan::base::{Base, BaseConfigBuilder};
use crate::backend::vulkan::errors::Error::ValidationLayerNotSupported;
use crate::backend::vulkan::utils::to_version;
use ash::vk;
use log::{LevelFilter, Metadata, Record};
use std::ffi::CString;
use std::iter::zip;
use std::sync::atomic::AtomicBool;

#[test]
fn test_base_config() {
    let base_cfg = BaseConfigBuilder::new().build("Test", "Test", "1.0.0", "1.0.0", "1.0.0");

    let entry = unsafe { ash::Entry::load() }.unwrap();
    if let Err((error_type)) = base_cfg.validate_layer_availability(&entry) {
        assert!(false, "Layer not found!");
    }
}

#[test]
fn test_base_config_fail() {
    let base_cfg = BaseConfigBuilder::new()
        .validation_layers(&["FAIL_LAYER"])
        .build("Test", "Test", "1.0.0", "1.0.0", "1.0.0");

    let entry = unsafe { ash::Entry::load() }.unwrap();
    if let Err(error) = base_cfg.validate_layer_availability(&entry) {
        match error {
            ValidationLayerNotSupported(index) => {
                assert_eq!(index, 0);
                println!("Layer error {}", index);
                return;
            }
        }
    }
    assert!(false, "Layer found!");
}
#[test]
fn test_base_config_fail_multiple() {
    let base_config = BaseConfigBuilder::new()
        .validation_layers(&["VK_LAYER_KHRONOS_validation", "VK_LAYER_KHRONOS_synchronization2", "FAIL_LAYER"])
        .build("Test", "Test", "1.0.0", "1.0.0", "1.0.0");

    let entry = unsafe { ash::Entry::load() }.unwrap();
    if let Err(error) = base_config.validate_layer_availability(&entry) {
        match error {
            ValidationLayerNotSupported(index) => {
                assert_eq!(index, 2);
                println!("Layer error {}", index);
                return;
            }
        }
    }
    assert!(false, "Layer found!");
}

#[test]
fn test_base_config_values() {
    let validation = "VK_LAYER_KHRONOS_validation";
    let sync = "VK_LAYER_KHRONOS_synchronization2";
    let validation_layers = &[validation, sync];
    let base_cfg = BaseConfigBuilder::new()
        .validation_layers(&[validation, sync])
        .build("Test1", "Test2", "1.3.0", "1.0.2", "1.0.3");

    assert_eq!(base_cfg.validation_layers.len(), 2);

    for (cstr_layer, layer) in zip(base_cfg.validation_layers.iter(), validation_layers.iter()) {
        let converted_layer = CString::new(*layer).unwrap();
        assert_eq!(*cstr_layer, converted_layer);
    }

    assert_eq!(base_cfg.application_name, CString::new("Test1").unwrap());
    assert_eq!(base_cfg.engine_name, CString::new("Test2").unwrap());
    assert_eq!(base_cfg.vulkan_api_version, vk::API_VERSION_1_3);
    assert_eq!(base_cfg.application_version, to_version("1.0.2"));
    assert_eq!(base_cfg.engine_version, to_version("1.0.3"));

    assert_eq!(base_cfg.vulkan_extensions, None);
}

struct TestLogger {
    was_called: AtomicBool,
}

impl log::Log for TestLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        true
    }

    fn log(
        &self,
        record: &Record,
    ) {
        self.was_called.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    fn flush(&self) {}
}

impl TestLogger {
    pub fn new() -> Self {
        Self {
            was_called: AtomicBool::new(false),
        }
    }
}
#[test]
fn vulkan_init_validation_test() {
    let test_logger = TestLogger::new();
    let test_logger_ptr = &test_logger as *const TestLogger;
    log::set_logger(unsafe { test_logger_ptr.as_ref().unwrap() }).expect("Failed to set logger");
    log::set_max_level(LevelFilter::Trace);

    let base_cfg = BaseConfigBuilder::new()
        .use_default_khr_validation()
        .build("Test", "Test", "1.3.0", "1.0.0", "1.0.0");
    Base::new(base_cfg).expect("Failed to create base");
    assert!(test_logger.was_called.load(std::sync::atomic::Ordering::Relaxed));
}
