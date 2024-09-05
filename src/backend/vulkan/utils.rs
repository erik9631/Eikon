use ash::vk;
use ash::vk::QueueFlags;

pub fn str_to_version(version: &str) -> u32 {
    let mut version = version.split(".");
    let major = version.next().unwrap().parse::<u32>().unwrap();
    let minor = version.next().unwrap().parse::<u32>().unwrap();
    let patch = version.next().unwrap().parse::<u32>().unwrap();
    vk::make_api_version(0, major, minor, patch)
}

pub const fn from_queue_flags_to_num(queues: QueueFlags, operations: Vec<u32>) -> u32 {
    match queues {
        QueueFlags::GRAPHICS => GRAPHICS_OP,
        QueueFlags::COMPUTE => COMPUTE_OP,
        QueueFlags::TRANSFER => TRANSFER_OP,
        QueueFlags::SPARSE_BINDING => SPARSE_BINDING_OP,
        QueueFlags::PROTECTED => PROTECTED_OP,
        _ => 0,
    }
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
