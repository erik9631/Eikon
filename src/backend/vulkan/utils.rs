use ash::vk;
use std::ffi::CString;
pub fn to_c_str(s: &str) -> CString {
    CString::new(s).unwrap()
}

#[macro_export]
macro_rules! platform_surface_extension {
    () => {{
        #[cfg(target_os = "windows")]
        {
            khr::win32_surface::NAME.as_ptr()
        }
        #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
        {
            #[cfg(feature = "wayland")]
            {
                khr::wayland_surface::NAME.as_ptr()
            }
            #[cfg(feature = "xlib")]
            {
                khr::xlib_surface::NAME.as_ptr()
            }
            #[cfg(feature = "xcb")]
            {
                khr::xcb_surface::NAME.as_ptr()
            }
        }
        #[cfg(target_os = "macos")]
        {
            ext::metal_surface::NAME.as_ptr()
        }
        #[cfg(target_os = "android")]
        {
            khr::android_surface::NAME.as_ptr()
        }
    }};
}

pub fn to_c_str_array<'a, I>(s: I) -> Vec<CString>
where
    I: Iterator<Item = &'a &'a str>,
{
    s.map(|x| to_c_str(x)).collect()
}
pub fn to_version(version: &str) -> u32 {
    let mut version = version.split(".");
    let major = version.next().unwrap().parse::<u32>().unwrap();
    let minor = version.next().unwrap().parse::<u32>().unwrap();
    let patch = version.next().unwrap().parse::<u32>().unwrap();
    vk::make_api_version(0, major, minor, patch)
}

pub const GRAPHICS_OP: u32 = 0b1;
pub const COMPUTE_OP: u32 = 0b10;
pub const TRANSFER_OP: u32 = 0b100;
pub const SPARSE_BINDING_OP: u32 = 0b1000;
pub const PROTECTED_OP: u32 = 0b00010000;
pub const VIDEO_DECODE_OP: u32 = 0b00100000;
pub const VIDEO_ENCODE_OP: u32 = 0b01000000;
pub const OPTICAL_FLOW_OP: u32 = 0b10000000;
pub const PRESENT_OP: u32 = 0x8000_0000;
