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
