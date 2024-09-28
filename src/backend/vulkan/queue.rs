use crate::backend::vulkan::queue::op_indices::COUNT;
use ash::vk;

pub mod op_indices {
    use ash::vk;

    pub const GRAPHICS: usize = 0;
    pub const COMPUTE: usize = 1;
    pub const TRANSFER: usize = 2;
    pub const PRESENT: usize = 3;
    pub const COUNT: usize = 4;

    pub fn queue_flags_to_op_index(flags: vk::QueueFlags) -> usize {
        match flags {
            vk::QueueFlags::GRAPHICS => GRAPHICS,
            vk::QueueFlags::COMPUTE => COMPUTE,
            vk::QueueFlags::TRANSFER => TRANSFER,
            _ => COUNT,
        }
    }
}

#[derive(Clone)]
pub struct QueueFamily {
    pub count: u32,
    pub priorities: Vec<f32>,
}

impl QueueFamily {
    pub fn new(count: u32, priorities: Vec<f32>) -> Self {
        Self { count, priorities }
    }
    pub fn single() -> Self {
        Self::new(1, vec![1.0])
    }
}

pub struct QueueHandles {
    pub queues: Vec<Option<vk::Queue>>,
}

impl QueueHandles {
    pub fn new() -> Self {
        Self { queues: vec![None; COUNT] }
    }
}

#[derive(Clone)]
pub struct QueueFamilyHandle {
    pub index: u32,
    pub offset: u32,
}

pub struct QueueSelections {
    pub families: Vec<Option<QueueFamily>>,           // Indexed based on family index
    pub operations: Vec<Option<(QueueFamilyHandle)>>, //Index, Offset --- Index based on operation, returns family index + offset
}

impl QueueSelections {
    pub fn new(family_count: u32) -> Self {
        Self {
            families: vec![None; family_count as usize],
            operations: vec![None; COUNT],
        }
    }

    pub fn insert_operation(&mut self, operation: u8, family_index: u32) -> Result<(), &str> {
        if self.operations[operation as usize].is_some() {
            return Err("Operation already exists!");
        }

        if let Some(handle) = self.families[family_index as usize].as_mut() {
            self.operations[operation as usize] = Some(QueueFamilyHandle {
                index: family_index,
                offset: handle.count,
            });
            handle.count += 1;
            /// TODO Parameters for priorities
            handle.priorities.push(1.0);
            return Ok(());
        }

        self.operations[operation as usize] = Some(QueueFamilyHandle {
            index: family_index,
            offset: 0,
        });

        self.families[family_index as usize] = Some(QueueFamily::single());
        Ok(())
    }

    pub fn to_vk_creation_info(&self) -> Vec<vk::DeviceQueueCreateInfo> {
        let mut queues = Vec::with_capacity(self.families.len());
        for (index, queue_family) in self.families.iter().enumerate() {
            if let Some(queue_family) = queue_family {
                queues.push(vk::DeviceQueueCreateInfo {
                    s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                    p_next: std::ptr::null(),
                    flags: Default::default(),
                    queue_family_index: index as u32,
                    queue_count: queue_family.count,
                    p_queue_priorities: queue_family.priorities.as_ptr(),
                    _marker: Default::default(),
                });
            }
        }
        queues
    }
}
