use crate::backend::vulkan::queue::indices::COUNT;
use ash::vk;

pub mod indices {
    pub const GRAPHICS: usize = 0;
    pub const COMPUTE: usize = 1;
    pub const TRANSFER: usize = 2;
    pub const PRESENT: usize = 3;
    pub const COUNT: usize = 4;
}

#[derive(Clone)]
pub struct QueueFamily {
    pub index: u32,
    pub count: u32,
    pub priorities: Vec<f32>,
}

impl QueueFamily {
    pub fn new(index: u32, count: u32, priorities: Vec<f32>) -> Self {
        Self { index, count, priorities }
    }
    pub fn single(index: u32) -> Self {
        Self::new(index, 1, vec![1.0])
    }
}

pub struct QueueHandles {
    pub queues: Vec<Option<Vec<vk::Queue>>>,
}

impl QueueHandles {
    pub fn new() -> Self {
        let mut handles = Self {
            queues: Vec::with_capacity(COUNT),
        };
        handles.queues.fill(None);
        handles
    }
}

pub struct QueueSelections {
    pub families: Vec<Option<QueueFamily>>,
}

impl QueueSelections {
    pub fn new() -> Self {
        let mut selections = Self {
            families: Vec::with_capacity(COUNT),
        };
        selections.families.fill(None);
        selections
    }
}
