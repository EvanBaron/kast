use crate::graphics::instance::QueueFamily;
use vk_bindings::*;

/// Holds the resources associated with a single frame in flight.
/// This allows for multiple frames to be processed concurrently (double/triple buffering)
/// without race conditions on the command buffers or synchronization primitives.
pub struct FrameData {
    pub command_pool: VkCommandPool,
    pub command_buffer: VkCommandBuffer,
    pub image_available_semaphore: VkSemaphore,
    pub render_finished_semaphore: VkSemaphore,
    pub in_flight_fence: VkFence,
}

impl FrameData {
    pub fn new(device: VkDevice, graphics_queue_family: QueueFamily) -> Self {
        let command_pool_create_info = VkCommandPoolCreateInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT,
            queueFamilyIndex: graphics_queue_family.family_index,
        };

        let mut command_pool = core::ptr::null_mut();
        unsafe {
            let result = vkCreateCommandPool(
                device,
                &command_pool_create_info,
                core::ptr::null_mut(),
                &mut command_pool,
            );
            if result != VK_SUCCESS {
                panic!("Failed to create command pool. Error: {:?}.", result);
            }
        }

        let command_buffer_allocate_info = VkCommandBufferAllocateInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            commandPool: command_pool,
            level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: 1,
        };

        let mut command_buffer = core::ptr::null_mut();
        unsafe {
            let result = vkAllocateCommandBuffers(
                device,
                &command_buffer_allocate_info,
                &mut command_buffer,
            );
            if result != VK_SUCCESS {
                panic!("Failed to allocate command buffer. Error: {:?}.", result);
            }
        }

        let semaphore_create_info = VkSemaphoreCreateInfo {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
        };

        let mut image_available_semaphore = core::ptr::null_mut();
        unsafe {
            let result = vkCreateSemaphore(
                device,
                &semaphore_create_info,
                core::ptr::null(),
                &mut image_available_semaphore,
            );
            if result != VK_SUCCESS {
                panic!("Failed to create semaphore. Error: {:?}.", result);
            }
        }

        let mut render_finished_semaphore = core::ptr::null_mut();
        unsafe {
            let result = vkCreateSemaphore(
                device,
                &semaphore_create_info,
                core::ptr::null(),
                &mut render_finished_semaphore,
            );
            if result != VK_SUCCESS {
                panic!("Failed to create semaphore. Error: {:?}.", result);
            }
        }

        let fence_create_info = VkFenceCreateInfo {
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: VK_FENCE_CREATE_SIGNALED_BIT,
        };

        let mut in_flight_fence = core::ptr::null_mut();
        unsafe {
            let result = vkCreateFence(
                device,
                &fence_create_info,
                core::ptr::null(),
                &mut in_flight_fence,
            );
            if result != VK_SUCCESS {
                panic!("Failed to create fence. Error: {:?}.", result);
            }
        }

        Self {
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
        }
    }

    pub fn destroy(&mut self, device: VkDevice) {
        unsafe {
            vkDestroySemaphore(device, self.image_available_semaphore, core::ptr::null());
            vkDestroySemaphore(device, self.render_finished_semaphore, core::ptr::null());
            vkDestroyFence(device, self.in_flight_fence, core::ptr::null());
            vkDestroyCommandPool(device, self.command_pool, core::ptr::null());
        }
    }
}
