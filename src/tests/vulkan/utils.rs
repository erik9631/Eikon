use crate::backend::vulkan::utils::to_version;
use ash::vk;

#[test]
fn test_str_to_version() {
    let result = vk::make_api_version(0, 1, 0, 0);
    assert_eq!(to_version("1.0.0"), result);

    let result = vk::make_api_version(0, 1, 2, 3);
    assert_eq!(to_version("1.2.3"), result);
}
