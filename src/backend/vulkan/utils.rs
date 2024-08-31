use ash::vk;
pub fn str_to_version(version: &str) -> u32 {
    let mut version = version.split(".");
    let major = version.next().unwrap().parse::<u32>().unwrap();
    let minor = version.next().unwrap().parse::<u32>().unwrap();
    let patch = version.next().unwrap().parse::<u32>().unwrap();
    vk::make_api_version(0, major, minor, patch)
}
