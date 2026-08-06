use vk_bindings::*;

use crate::backend::vulkan::{
    allocator::{Allocation, VulkanAllocator},
    command::VulkanCommandBuffer,
    deletion_queue::DeletionQueue,
};

/// A Vulkan buffer with associated device memory.
///
/// Manages GPU memory allocation via the custom allocator and provides methods
/// for data transfer, memory mapping, and buffer-to-buffer copies.
pub struct VulkanBuffer {
    pub handle: VkBuffer,
    pub allocation: Allocation,
    pub size: VkDeviceSize,
    device: VkDevice,
}

impl VulkanBuffer {
    /// Creates a new buffer with the specified size, usage, and memory properties.
    ///
    /// # Arguments
    /// * `allocator` - The memory allocator to use
    /// * `device` - The Vulkan device
    /// * `size` - The size of the buffer in bytes
    /// * `usage` - Buffer usage flags
    /// * `memory_properties` - Memory property flags
    ///
    /// # Returns
    /// A new VulkanBuffer or an error if creation or memory allocation fails.
    pub fn new(
        allocator: &mut VulkanAllocator,
        device: VkDevice,
        size: u64,
        usage: VkBufferUsageFlags,
        memory_properties: VkMemoryPropertyFlags,
    ) -> Result<Self, String> {
        let create_info = VkBufferCreateInfo {
            sType: VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            size,
            usage,
            sharingMode: VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateBuffer(device, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create buffer: {}", result));
            }
        }

        // Get memory requirements
        let mut memory_requirements = VkMemoryRequirements::default();
        unsafe {
            vkGetBufferMemoryRequirements(device, handle, &mut memory_requirements);
        }

        // Allocate memory using the allocator
        let allocation = allocator
            .allocate(
                memory_requirements.size,
                memory_requirements.alignment,
                memory_requirements.memoryTypeBits,
                memory_properties,
            )
            .map_err(|e| {
                unsafe {
                    vkDestroyBuffer(device, handle, core::ptr::null());
                }
                e
            })?;

        // Bind buffer to memory
        unsafe {
            let result = vkBindBufferMemory(device, handle, allocation.memory, allocation.offset);
            if result != VK_SUCCESS {
                allocator.free(&allocation);
                vkDestroyBuffer(device, handle, core::ptr::null());
                return Err(format!("Failed to bind buffer memory: {}", result));
            }
        }

        Ok(Self {
            handle,
            allocation,
            size,
            device: device,
        })
    }

    /// Maps the buffer's memory into CPU-accessible address space.
    ///
    /// The buffer must have been created with HOST_VISIBLE memory property.
    ///
    /// # Arguments
    /// * `allocator` - The allocator that owns this buffer's memory
    ///
    /// # Returns
    /// A pointer to the mapped memory or an error if mapping fails.
    pub fn map(&self, allocator: &VulkanAllocator) -> Result<*mut core::ffi::c_void, String> {
        allocator.map(&self.allocation)
    }

    /// Unmaps the buffer's memory from CPU-accessible address space.
    ///
    /// # Arguments
    /// * `allocator` - The allocator that owns this buffer's memory
    pub fn unmap(&self, allocator: &VulkanAllocator) {
        allocator.unmap(&self.allocation);
    }

    /// Writes data to the buffer. The buffer must be host-visible.
    ///
    /// # Arguments
    /// * `allocator` - The allocator that owns this buffer's memory
    /// * `data` - The data to write
    ///
    /// # Safety
    /// This function assumes the buffer has HOST_VISIBLE memory property.
    pub fn write_data<T: Copy>(
        &self,
        allocator: &VulkanAllocator,
        data: &[T],
    ) -> Result<(), String> {
        let size = (core::mem::size_of::<T>() * data.len()) as u64;

        if size > self.size {
            return Err("Data exceeds buffer size".to_string());
        }

        let mapped = self.map(allocator)?;
        unsafe {
            core::ptr::copy_nonoverlapping(data.as_ptr(), mapped as *mut T, data.len());
        }

        self.unmap(allocator);

        Ok(())
    }

    /// Copies data from this buffer to another buffer using a command buffer.
    ///
    /// # Arguments
    /// * `command_buffer` - The command buffer to record the copy command into
    /// * `dst` - The destination buffer
    /// * `src_offset` - The source offset in bytes
    /// * `dst_offset` - The destination offset in bytes
    /// * `size` - The number of bytes to copy
    pub fn copy_to(
        &self,
        command_buffer: &VulkanCommandBuffer,
        dst: &VulkanBuffer,
        src_offset: u64,
        dst_offset: u64,
        size: u64,
    ) {
        let region = VkBufferCopy {
            srcOffset: src_offset,
            dstOffset: dst_offset,
            size,
        };

        unsafe {
            vkCmdCopyBuffer(command_buffer.handle, self.handle, dst.handle, 1, &region);
        }
    }

    /// Destroys the buffer by scheduling cleanup via the deletion queue.
    ///
    /// This is the recommended way to destroy buffers when using multiple frames in flight.
    /// The Vulkan handle will be destroyed when the current frame completes, and the
    /// memory allocation will be freed automatically.
    ///
    /// # Arguments
    /// * `deletion_queue` - The deletion queue to schedule cleanup with
    pub fn destroy(self, deletion_queue: &mut DeletionQueue) {
        let handle = self.handle;
        let allocation = self.allocation.clone();
        let device = self.device;
        core::mem::forget(self);

        deletion_queue.push_with_allocation(allocation, move || unsafe {
            vkDestroyBuffer(device, handle, core::ptr::null());
        });
    }

    /// Destroys the buffer immediately.
    /// Only safe when the GPU is known to be idle.
    ///
    /// # Arguments
    /// * `allocator` - The Vulkan allocator to free the memory allocation with
    pub fn destroy_immediate(self, allocator: &mut VulkanAllocator) {
        let handle = self.handle;
        let allocation = self.allocation.clone();
        let device = self.device;
        core::mem::forget(self);
        allocator.free(&allocation);
        unsafe {
            vkDestroyBuffer(device, handle, core::ptr::null());
        }
    }
}

impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!(
                "VulkanBuffer must not be dropped directly. Use destroy() with a deletion queue."
            );
        }
    }
}
