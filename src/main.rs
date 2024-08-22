use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::ptr;
use std::str::{from_utf8, from_utf8_unchecked};
use ash::{ext, khr, vk};
use winit::application::ApplicationHandler;
use winit::event::{Event, WindowEvent};
use winit::event::WindowEvent::{CloseRequested, Resized};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::{Window, WindowId};

const WINDOW_TITLE: &'static str = "01.Instance Creation";
const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
const ENGINE_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);
const API_VERSION: u32 = vk::API_VERSION_1_3;
const REQUIRED_EXTENSION_NAMES: &[*const c_char] = &[
    khr::surface::NAME.as_ptr(),
    khr::win32_surface::NAME.as_ptr(),
    ext::debug_utils::NAME.as_ptr(),
];

fn create_validation_layers() -> HashMap<&'static str, (&'static str, bool)> {
    let mut layers = HashMap::new();
    layers.insert("VK_LAYER_KHRONOS_validation", ("VK_LAYER_KHRONOS_validation\0", true));
    layers
}

unsafe extern "system" fn debug_callback(message_severity: vk::DebugUtilsMessageSeverityFlagsEXT , message_type: vk::DebugUtilsMessageTypeFlagsEXT, p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT , _p_user_data: *mut c_void ) -> vk::Bool32 {
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

fn create_messenger_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static>{
    vk::DebugUtilsMessengerCreateInfoEXT{
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::WARNING | vk::DebugUtilsMessageSeverityFlagsEXT::ERROR | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | vk::DebugUtilsMessageTypeFlagsEXT::GENERAL,
        p_next: ptr::null(),
        pfn_user_callback: Some(debug_callback),
        p_user_data: ptr::null_mut(),
        _marker: Default::default(),
    }
}

fn create_vulcan_instance(entry: & ash::Entry, validation_layers: Vec<*const c_char>, debug_struct_info: *const vk::DebugUtilsMessengerCreateInfoEXT) -> ash::Instance {
    let application_info = vk::ApplicationInfo {
        s_type: vk::StructureType::APPLICATION_INFO,
        p_next: ptr::null(),
        p_application_name: WINDOW_TITLE.as_ptr() as *const i8,
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

    unsafe {entry.create_instance(&create_info, None).expect("Failed to create instance!")}
}

fn init_swap_chain() {}

fn init_commands() {}

fn init_sync() {}


pub struct VulcanApp {
    window: Option<Window>,
    ash_entry: ash::Entry,
    instance: ash::Instance,
}

impl Drop for VulcanApp {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
        }
    }
    
}

impl VulcanApp {


    fn verify_validation_layers(entry: &ash::Entry, mut layers: HashMap<&'static str, (&'static str, bool)>) -> Vec<*const c_char> {
        let mut vec = Vec::new();
        let properties = unsafe {entry.enumerate_instance_layer_properties().expect("Failed to enumerate instance layer properties")};
        if properties.len() == 0 {
            panic!("No validation layers found");
        }

        for layer in properties{
            let layer_string = unsafe {from_utf8_unchecked(CStr::from_ptr(layer.layer_name.as_ptr()).to_bytes())};

            if let Some(layer) = layers.remove(layer_string){
                if layer.1 {
                    vec.push(layer.0.as_ptr() as *const c_char);
                }
            }
        }

        if layers.len() != 0 {
            for layer in layers{
                println!("Layer {} not found", layer.0);
            }
            panic!("Validation layers not found!");
        }
        vec
    }

    pub fn new(validation_layers: HashMap<&'static str, (&'static str, bool)>) -> Self {
        let mut event_loop_builder = EventLoop::builder();
        let event_loop= event_loop_builder.with_any_thread(true).build().expect("Failed to create event loop");
        let ash_entry = unsafe {ash::Entry::load().expect("Failed to create entry")};

        let layers = Self::verify_validation_layers(&ash_entry, validation_layers);
        let debug_struct_info = create_messenger_info();
        println!("{}", layers.len());

        let instance = create_vulcan_instance(&ash_entry, layers, &debug_struct_info as *const vk::DebugUtilsMessengerCreateInfoEXT);

        let mut app = Self { ash_entry,instance, window: None };
        event_loop.run_app(&mut app).expect("Failed to run event loop");
        app
    }
}
impl ApplicationHandler for VulcanApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(Window::default_attributes()).expect("Failed to create window");
        window.set_title("Eikon Engine");
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        match event {
            CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            Resized(data) => {
                println!("Window resized to {} {}", data.width, data.height);
            }
            _ => {}
        }
    }
}
fn main() {
    let validation_layers = create_validation_layers();
    let app = VulcanApp::new(validation_layers);
}
