use vk_bindings::*;

/// A Vulkan semaphore for GPU-GPU synchronization.
///
/// Semaphores are used to coordinate operations between queue submissions and
/// between queues. They are typically used for image acquisition and presentation.
pub struct VulkanSemaphore {
    pub(crate) handle: VkSemaphore,
    device: VkDevice,
}

impl VulkanSemaphore {
    /// Creates a new Vulkan semaphore.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device handle.
    pub fn new(device: VkDevice) -> Result<Self, String> {
        let create_info = VkSemaphoreCreateInfo {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateSemaphore(device, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create semaphore: {}", result));
            }
        }

        Ok(Self { handle, device })
    }
}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unsafe {
            vkDestroySemaphore(self.device, self.handle, core::ptr::null());
        }
    }
}

/// A Vulkan fence for GPU-CPU synchronization.
///
/// Fences are used to synchronize between the GPU and CPU. They allow the CPU
/// to wait for GPU operations to complete and can be queried for their status.
pub struct VulkanFence {
    pub(crate) handle: VkFence,
    device: VkDevice,
}

impl VulkanFence {
    /// Creates a new Vulkan fence.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device handle.
    /// * `signaled` - If true, the fence is created in the signaled state.
    pub fn new(device: VkDevice, signaled: bool) -> Result<Self, String> {
        let flags = if signaled {
            VK_FENCE_CREATE_SIGNALED_BIT
        } else {
            0
        };

        let create_info = VkFenceCreateInfo {
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateFence(device, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create fence: {}", result));
            }
        }

        Ok(Self { handle, device })
    }

    /// Waits for the fence to be signaled.
    ///
    /// # Arguments
    /// * `timeout_ns` - Timeout in nanoseconds. Use u64::MAX for no timeout.
    pub fn wait(&self, timeout_ns: u64) -> Result<(), String> {
        unsafe {
            let result = vkWaitForFences(self.device, 1, &self.handle, VK_TRUE, timeout_ns);
            if result != VK_SUCCESS {
                return Err(format!("Failed to wait for fence: {}", result));
            }
        }
        Ok(())
    }

    /// Resets the fence to the unsignaled state.
    pub fn reset(&self) -> Result<(), String> {
        unsafe {
            let result = vkResetFences(self.device, 1, &self.handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to reset fence: {}", result));
            }
        }
        Ok(())
    }

    /// Checks if the fence is currently signaled.
    pub fn is_signaled(&self) -> bool {
        unsafe { vkGetFenceStatus(self.device, self.handle) == VK_SUCCESS }
    }
}

impl Drop for VulkanFence {
    fn drop(&mut self) {
        unsafe {
            vkDestroyFence(self.device, self.handle, core::ptr::null());
        }
    }
}
