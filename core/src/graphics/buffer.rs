use vk_bindings::*;
use crate::graphics::utils;

pub struct AllocatedBuffer {
    pub handle: VkBuffer,
    pub memory: VkDeviceMemory,
    pub size: VkDeviceSize,
}

impl AllocatedBuffer {
    /// Creates a new buffer with the given properties.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device to create the buffer on.
    /// * `physical_device` - The physical device to create the buffer on.
    /// * `size` - The size of the buffer in bytes.
    /// * `usage` - The usage flags for the buffer.
    /// * `properties` - The memory property flags for the buffer.
    pub fn new(
        device: VkDevice,
        physical_device: VkPhysicalDevice,
        size: VkDeviceSize,
        usage: VkBufferUsageFlags,
        properties: VkMemoryPropertyFlags,
    ) -> Self {
        let buffer_create_info = VkBufferCreateInfo {
            sType: VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            size,
            usage,
            sharingMode: VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
        };

        let mut buffer = core::ptr::null_mut();
        let result =
            unsafe { vkCreateBuffer(device, &buffer_create_info, core::ptr::null(), &mut buffer) };

        if result != VK_SUCCESS {
            panic!("Failed to create buffer. Error: {}.", result);
        }

        let mut memory_requirements = VkMemoryRequirements::default();
        unsafe {
            vkGetBufferMemoryRequirements(device, buffer, &mut memory_requirements);
        };

        let memory_type_index = utils::find_memory_type(
            physical_device,
            memory_requirements.memoryTypeBits,
            properties,
        );

        let memory_allocation_info = VkMemoryAllocateInfo {
            sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            allocationSize: memory_requirements.size,
            memoryTypeIndex: memory_type_index,
        };

        let mut buffer_memory = core::ptr::null_mut();
        let result = unsafe {
            vkAllocateMemory(
                device,
                &memory_allocation_info,
                core::ptr::null(),
                &mut buffer_memory,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to allocate buffer memory. Error: {}.", result);
        };

        let result = unsafe { vkBindBufferMemory(device, buffer, buffer_memory, 0) };

        if result != VK_SUCCESS {
            panic!("Failed to bind buffer memory. Error: {}.", result);
        };

        Self {
            handle: buffer,
            memory: buffer_memory,
            size,
        }
    }

    pub fn destroy(&self, device: VkDevice) {
        unsafe {
            vkDestroyBuffer(device, self.handle, core::ptr::null());
            vkFreeMemory(device, self.memory, core::ptr::null());
        }
    }
}
