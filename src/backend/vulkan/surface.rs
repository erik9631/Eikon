use crate::backend::vulkan::base::Base;
use crate::{fatal_assert, fatal_unwrap, fatal_unwrap_e};
use ash::vk::PhysicalDevice;
use ash::{khr, vk};
use log::{error, info, trace, warn};
use std::ptr::null;
use winit::raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub struct Surface {
    raw_window_handle: RawWindowHandle,
    raw_display_handle: RawDisplayHandle,
    surface_instance: khr::surface::Instance,
    pub surface: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(base: &Base, raw_window_handle: RawWindowHandle, raw_display_handle: RawDisplayHandle) -> Self {
        let surface_instance = khr::surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
        let surface = Self::create_surface(base, raw_window_handle, raw_display_handle);
        Self {
            surface_instance,
            surface,
            raw_window_handle,
            raw_display_handle,
        }
    }
    fn create_surface(base: &Base, raw_window_handle: RawWindowHandle, raw_display_handle: RawDisplayHandle) -> vk::SurfaceKHR {
        let raw_window_handle = raw_window_handle;
        match raw_window_handle {
            /// Win32
            RawWindowHandle::Win32(raw_handle) => {
                let win32_surface_loader = khr::win32_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let surface_info = vk::Win32SurfaceCreateInfoKHR {
                    s_type: vk::StructureType::WIN32_SURFACE_CREATE_INFO_KHR,
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
                platform_surface
            }

            /// Linux
            RawWindowHandle::Wayland(raw_handle) => {
                let wayland_surface_loader = khr::wayland_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let display = match raw_display_handle {
                    RawDisplayHandle::Wayland(display) => display.display,
                    _ => fatal_assert!("Wayland surfaces must be created with a Wayland display handle!"),
                };

                let surface_info = vk::WaylandSurfaceCreateInfoKHR {
                    s_type: vk::StructureType::WAYLAND_SURFACE_CREATE_INFO_KHR,
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
                platform_surface
            }
            RawWindowHandle::Xcb(raw_handle) => {
                let xcb_surface_loader = khr::xcb_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let mut display = match raw_display_handle {
                    RawDisplayHandle::Xcb(display) => display,
                    _ => fatal_assert!("XCB surfaces must be created with a XCB display handle!"),
                };

                let surface_info = vk::XcbSurfaceCreateInfoKHR {
                    s_type: vk::StructureType::XCB_SURFACE_CREATE_INFO_KHR,
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
                platform_surface
            }
            RawWindowHandle::Xlib(raw_handle) => {
                let xlib_surface_loader = khr::xlib_surface::Instance::new(&base.ash_instance, &base.vulkan_instance);
                let display = match raw_display_handle {
                    RawDisplayHandle::Xlib(display) => display,
                    _ => fatal_assert!("Xlib surfaces must be created with a Xlib display handle!"),
                };

                let surface_info = vk::XlibSurfaceCreateInfoKHR {
                    s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
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
                platform_surface
            }

            (_) => {
                fatal_assert!("Unsupported window handle type!");
            }
        }
    }

    pub fn get_physical_device_surface_capabilities(&self, physical_device: &PhysicalDevice) -> vk::SurfaceCapabilitiesKHR {
        unsafe {
            fatal_unwrap_e!(
                self.surface_instance
                    .get_physical_device_surface_capabilities(*physical_device, self.surface),
                "Failed to get surface capabilities! {}"
            )
        }
    }

    pub fn get_physical_device_surface_formats(&self, physical_device: &PhysicalDevice) -> Vec<vk::SurfaceFormatKHR> {
        unsafe {
            fatal_unwrap_e!(
                self.surface_instance
                    .get_physical_device_surface_formats(*physical_device, self.surface),
                "Failed to get surface formats! {}"
            )
        }
    }

    pub fn get_physical_device_surface_present_modes(&self, physical_device: &PhysicalDevice) -> Vec<vk::PresentModeKHR> {
        unsafe {
            fatal_unwrap_e!(
                self.surface_instance
                    .get_physical_device_surface_present_modes(*physical_device, self.surface),
                "Failed to get surface present modes! {}"
            )
        }
    }

    pub fn get_physical_device_surface_support(&self, physical_device: PhysicalDevice, queue_family_index: u32) -> bool {
        unsafe {
            fatal_unwrap_e!(
                self.surface_instance
                    .get_physical_device_surface_support(physical_device, queue_family_index, self.surface),
                "Failed to get surface support! {}"
            )
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.surface_instance.destroy_surface(self.surface, None);
        }
    }
}
