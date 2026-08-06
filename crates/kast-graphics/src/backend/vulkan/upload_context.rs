use vk_bindings::*;

use crate::backend::vulkan::{
    allocator::VulkanAllocator, buffer::VulkanBuffer, command::VulkanCommandBuffer,
};

/// Sentinel value meaning "this buffer is safe to reuse" set by recycle_staging_buffers()
/// after the owning frame's fence has been waited on.
const FRAME_RECYCLED: usize = usize::MAX;

pub struct StagingEntry {
    pub buffer: VulkanBuffer,
    pub frame_index: usize,
}

/// Context for asynchronous resource uploads.
///
/// Manages command buffers, fences, and staging buffers for uploading data
/// to GPU resources without blocking rendering. Uses a ring buffer approach
/// with one command buffer per frame in flight.
pub struct UploadContext {
    pub staging_entries: Vec<StagingEntry>,
    device: VkDevice,
    command_pool: VkCommandPool,
    command_buffers: Vec<VkCommandBuffer>,
    fences: Vec<VkFence>,
    current_frame: usize,
    frames_in_flight: usize,
}

impl UploadContext {
    /// Creates a new upload context.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device
    /// * `queue_family_index` - The queue family to allocate command buffers from
    /// * `frames_in_flight` - Number of frames that can be in flight simultaneously
    ///
    /// # Returns
    /// A new UploadContext or an error if creation fails
    pub fn new(
        device: VkDevice,
        queue_family_index: u32,
        frames_in_flight: usize,
    ) -> Result<Self, String> {
        assert!(
            frames_in_flight > 0,
            "frames_in_flight must be greater than 0"
        );

        let pool_create_info = VkCommandPoolCreateInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
            queueFamilyIndex: queue_family_index,
        };

        let mut command_pool = core::ptr::null_mut();
        unsafe {
            let result = vkCreateCommandPool(
                device,
                &pool_create_info,
                core::ptr::null(),
                &mut command_pool,
            );
            if result != VK_SUCCESS {
                return Err(format!("Failed to create upload command pool: {}", result));
            }
        }

        let allocate_info = VkCommandBufferAllocateInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            commandPool: command_pool,
            level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: frames_in_flight as u32,
        };

        let mut command_buffers = vec![core::ptr::null_mut(); frames_in_flight];
        unsafe {
            let result =
                vkAllocateCommandBuffers(device, &allocate_info, command_buffers.as_mut_ptr());

            if result != VK_SUCCESS {
                vkDestroyCommandPool(device, command_pool, core::ptr::null());
                return Err(format!(
                    "Failed to allocate upload command buffers: {}",
                    result
                ));
            }
        }

        let mut fences = Vec::with_capacity(frames_in_flight);
        let fence_create_info = VkFenceCreateInfo {
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: VK_FENCE_CREATE_SIGNALED_BIT,
        };

        for _ in 0..frames_in_flight {
            let mut fence = core::ptr::null_mut();
            unsafe {
                let result =
                    vkCreateFence(device, &fence_create_info, core::ptr::null(), &mut fence);

                if result != VK_SUCCESS {
                    for &f in &fences {
                        vkDestroyFence(device, f, core::ptr::null());
                    }

                    vkDestroyCommandPool(device, command_pool, core::ptr::null());
                    return Err(format!("Failed to create upload fence: {}", result));
                }
            }

            fences.push(fence);
        }

        Ok(Self {
            device,
            command_pool,
            command_buffers,
            fences,
            staging_entries: Vec::new(),
            current_frame: 0,
            frames_in_flight,
        })
    }

    /// Begins a new upload command buffer for the current frame.
    ///
    /// Waits for the current frame's fence (ensuring previous uploads are complete),
    /// then begins recording commands.
    ///
    /// # Returns
    /// The command buffer ready for recording, or an error
    pub fn begin(&mut self) -> Result<VkCommandBuffer, String> {
        let fence = self.fences[self.current_frame];
        let command_buffer = self.command_buffers[self.current_frame];

        // Wait for previous upload on this frame to complete
        unsafe {
            let result = vkWaitForFences(self.device, 1, &fence, VK_TRUE, u64::MAX);
            if result != VK_SUCCESS {
                return Err(format!("Failed to wait for upload fence: {}", result));
            }

            let result = vkResetFences(self.device, 1, &fence);
            if result != VK_SUCCESS {
                return Err(format!("Failed to reset upload fence: {}", result));
            }
        }

        self.recycle_staging_buffers();

        let begin_info = VkCommandBufferBeginInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
            pInheritanceInfo: core::ptr::null(),
        };

        unsafe {
            let result = vkBeginCommandBuffer(command_buffer, &begin_info);
            if result != VK_SUCCESS {
                return Err(format!("Failed to begin upload command buffer: {}", result));
            }
        }

        Ok(command_buffer)
    }

    /// Ends and submits the current frame's upload command buffer.
    ///
    /// # Arguments
    /// * `queue` - The queue to submit to
    ///
    /// # Returns
    /// Ok(()) if submission succeeds, or an error
    pub fn submit(&mut self, queue: VkQueue) -> Result<(), String> {
        let command_buffer = self.command_buffers[self.current_frame];
        let fence = self.fences[self.current_frame];

        unsafe {
            let result = vkEndCommandBuffer(command_buffer);
            if result != VK_SUCCESS {
                return Err(format!("Failed to end upload command buffer: {}", result));
            }

            let submit_info = VkSubmitInfo {
                sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
                pNext: core::ptr::null(),
                waitSemaphoreCount: 0,
                pWaitSemaphores: core::ptr::null(),
                pWaitDstStageMask: core::ptr::null(),
                commandBufferCount: 1,
                pCommandBuffers: &command_buffer,
                signalSemaphoreCount: 0,
                pSignalSemaphores: core::ptr::null(),
            };

            let result = vkQueueSubmit(queue, 1, &submit_info, fence);
            if result != VK_SUCCESS {
                return Err(format!(
                    "Failed to submit upload command buffer: {}",
                    result
                ));
            }
        }

        Ok(())
    }

    /// Advances to the next frame.
    ///
    /// Call this at the beginning of each frame.
    pub fn next_frame(&mut self) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
    }

    /// Checks if the current frame's upload is complete.
    ///
    /// # Returns
    /// true if complete, false if still in progress
    pub fn is_complete(&self) -> bool {
        let fence = self.fences[self.current_frame];
        unsafe { vkGetFenceStatus(self.device, fence) == VK_SUCCESS }
    }

    /// Allocates or retrieves a staging buffer of at least the given size.
    ///
    /// # Arguments
    /// * `allocator` - The memory allocator
    /// * `size` - Minimum size required
    ///
    /// # Returns
    /// Index of a suitable staging buffer or an error
    pub fn get_staging_buffer_index(
        &mut self,
        allocator: &mut VulkanAllocator,
        size: u64,
    ) -> Result<usize, String> {
        // Only reuse a buffer that has been explicitly recycled (fence was waited on).
        for (i, entry) in self.staging_entries.iter().enumerate() {
            if entry.frame_index == FRAME_RECYCLED && entry.buffer.size >= size {
                return Ok(i);
            }
        }

        // No suitable recycled buffer, allocate a new one.
        let rounded_size = size.next_power_of_two().max(64 * 1024);
        let buffer = VulkanBuffer::new(
            allocator,
            self.device,
            rounded_size,
            VK_BUFFER_USAGE_TRANSFER_SRC_BIT,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
        )?;
        self.staging_entries.push(StagingEntry {
            frame_index: FRAME_RECYCLED,
            buffer,
        });

        Ok(self.staging_entries.len() - 1)
    }

    /// Uploads data to a buffer using a staging buffer.
    ///
    /// # Arguments
    /// * `allocator` - The memory allocator
    /// * `command_buffer` - The command buffer to record into (from begin())
    /// * `dst_buffer` - The destination buffer
    /// * `data` - The data to upload
    ///
    /// # Returns
    /// Ok(()) on success, or an error
    pub fn upload_to_buffer(
        &mut self,
        allocator: &mut VulkanAllocator,
        command_buffer: &VulkanCommandBuffer,
        dst_buffer: &VulkanBuffer,
        data: &[u8],
    ) -> Result<(), String> {
        let staging_index = self.get_staging_buffer_index(allocator, data.len() as u64)?;

        // Mark as in-use by the current frame before writing so recycling can't
        // hand it out again until the fence for this frame fires.
        self.staging_entries[staging_index].frame_index = self.current_frame;
        let staging_buffer = &self.staging_entries[staging_index].buffer;
        staging_buffer.write_data(allocator, data)?;

        staging_buffer.copy_to(command_buffer, &dst_buffer, 0, 0, data.len() as u64);

        Ok(())
    }

    /// Waits for all uploads to complete.
    /// This is useful during shutdown or when you need to ensure all uploads are done.
    pub fn wait_idle(&self) -> Result<(), String> {
        unsafe {
            let result = vkWaitForFences(
                self.device,
                self.fences.len() as u32,
                self.fences.as_ptr(),
                VK_TRUE,
                u64::MAX,
            );
            if result != VK_SUCCESS {
                return Err(format!("Failed to wait for upload fences: {}", result));
            }
        }

        Ok(())
    }

    /// Marks all staging buffers that were used by the current frame as safe to reuse.
    /// This must only be called after the fence for `current_frame` has been waited on,
    /// which `begin()` guarantees.
    fn recycle_staging_buffers(&mut self) {
        for entry in &mut self.staging_entries {
            if entry.frame_index == self.current_frame {
                entry.frame_index = FRAME_RECYCLED;
            }
        }
    }

    /// Returns the current frame index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn destroy(&mut self, allocator: &mut VulkanAllocator) {
        unsafe {
            let _ = self.wait_idle();

            for entry in self.staging_entries.drain(..) {
                entry.buffer.destroy_immediate(allocator);
            }

            for &fence in &self.fences {
                vkDestroyFence(self.device, fence, core::ptr::null());
            }

            vkDestroyCommandPool(self.device, self.command_pool, core::ptr::null());
        }
    }
}

impl Drop for UploadContext {
    fn drop(&mut self) {
        if !self.staging_entries.is_empty() {
            if !std::thread::panicking() {
                panic!("UploadContext was dropped without calling destroy()! GPU memory leaked.");
            }
        }
    }
}
