use crate::graphics::buffer::AllocatedBuffer;
use crate::graphics::descriptors::DescriptorPool;
use crate::graphics::instance::QueueFamily;
use crate::graphics::uniforms::{CameraData, ObjectData};
use vk_bindings::*;

const MAX_OBJECT: usize = 10_000;

/// Holds the resources associated with a single frame in flight.
pub struct FrameData {
    pub command_pool: VkCommandPool,
    pub command_buffer: VkCommandBuffer,
    pub image_available_semaphore: VkSemaphore,
    pub render_finished_semaphore: VkSemaphore,
    pub in_flight_fence: VkFence,
    pub present_fence: VkFence,
    pub global_buffer: AllocatedBuffer,
    pub object_buffer: AllocatedBuffer,
    pub descriptor_set: VkDescriptorSet,
    pub delete_queue_framebuffers: Vec<VkFramebuffer>,
    pub delete_queue_image_views: Vec<VkImageView>,
    pub global_buffer_mapped: *mut core::ffi::c_void,
    pub object_buffer_mapped: *mut core::ffi::c_void,
}

impl FrameData {
    /// Creates a new FrameData instance.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `physical_device` - The Vulkan physical device.
    /// * `graphics_queue_family` - The graphics queue family.
    /// * `descriptor_pool` - The descriptor pool.
    /// * `descriptor_set_layout` - The descriptor set layout.
    pub fn new(
        device: VkDevice,
        physical_device: VkPhysicalDevice,
        graphics_queue_family: QueueFamily,
        descriptor_pool: &DescriptorPool,
        descriptor_set_layout: VkDescriptorSetLayout,
    ) -> Self {
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

        // Global Buffer
        let global_buffer_size = core::mem::size_of::<CameraData>() as u64;
        let global_buffer = AllocatedBuffer::new(
            device,
            physical_device,
            global_buffer_size,
            VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT as VkBufferUsageFlags,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags
                | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT as VkMemoryPropertyFlags,
        );

        let mut global_buffer_mapped = core::ptr::null_mut();
        unsafe {
            let result = vkMapMemory(
                device,
                global_buffer.memory,
                0,
                global_buffer_size,
                0,
                &mut global_buffer_mapped,
            );
            if result != VK_SUCCESS {
                panic!("Failed to map global buffer memory. Error: {}", result);
            }
        }

        // Object Buffer
        let object_buffer_size = (MAX_OBJECT * core::mem::size_of::<ObjectData>()) as u64;

        let object_buffer = AllocatedBuffer::new(
            device,
            physical_device,
            object_buffer_size,
            VK_BUFFER_USAGE_STORAGE_BUFFER_BIT as VkBufferUsageFlags,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags
                | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT as VkMemoryPropertyFlags,
        );

        let mut object_buffer_mapped = core::ptr::null_mut();
        unsafe {
            let result = vkMapMemory(
                device,
                object_buffer.memory,
                0,
                object_buffer_size,
                0,
                &mut object_buffer_mapped,
            );
            if result != VK_SUCCESS {
                panic!("Failed to map object buffer memory. Error: {}", result);
            }
        }

        // Allocate Descriptor Set
        let descriptor_set = descriptor_pool.allocate_set(device, descriptor_set_layout);

        // Update Descriptor Set
        let global_buffer_info = VkDescriptorBufferInfo {
            buffer: global_buffer.handle,
            offset: 0,
            range: global_buffer_size,
        };

        let global_write = VkWriteDescriptorSet {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: descriptor_set,
            dstBinding: 0,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            pImageInfo: core::ptr::null(),
            pBufferInfo: &global_buffer_info,
            pTexelBufferView: core::ptr::null(),
        };

        // Update Descriptor Set
        let object_buffer_info = VkDescriptorBufferInfo {
            buffer: object_buffer.handle,
            offset: 0,
            range: object_buffer_size,
        };

        let object_write = VkWriteDescriptorSet {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: descriptor_set,
            dstBinding: 1,
            dstArrayElement: 0,
            descriptorCount: 1,
            descriptorType: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
            pImageInfo: core::ptr::null(),
            pBufferInfo: &object_buffer_info,
            pTexelBufferView: core::ptr::null(),
        };

        let descriptor_writes = [global_write, object_write];

        unsafe {
            vkUpdateDescriptorSets(
                device,
                descriptor_writes.len() as u32,
                descriptor_writes.as_ptr(),
                0,
                core::ptr::null(),
            );
        }

        // Sync primitives
        let semaphore_create_info = VkSemaphoreCreateInfo {
            sType: VK_STRUCTURE_TYPE_SEMAPHORE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
        };

        let mut image_available_semaphore = core::ptr::null_mut();
        let mut render_finished_semaphore = core::ptr::null_mut();
        unsafe {
            vkCreateSemaphore(
                device,
                &semaphore_create_info,
                core::ptr::null(),
                &mut image_available_semaphore,
            );
            vkCreateSemaphore(
                device,
                &semaphore_create_info,
                core::ptr::null(),
                &mut render_finished_semaphore,
            );
        }

        let fence_create_info = VkFenceCreateInfo {
            sType: VK_STRUCTURE_TYPE_FENCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: VK_FENCE_CREATE_SIGNALED_BIT,
        };

        let mut in_flight_fence = core::ptr::null_mut();
        let mut present_fence = core::ptr::null_mut();
        unsafe {
            vkCreateFence(
                device,
                &fence_create_info,
                core::ptr::null(),
                &mut in_flight_fence,
            );
            vkCreateFence(
                device,
                &fence_create_info,
                core::ptr::null(),
                &mut present_fence,
            );
        }

        let delete_queue_framebuffers = Vec::new();
        let delete_queue_image_views = Vec::new();

        Self {
            command_pool,
            command_buffer,
            image_available_semaphore,
            render_finished_semaphore,
            in_flight_fence,
            present_fence,
            global_buffer,
            object_buffer,
            descriptor_set,
            global_buffer_mapped,
            object_buffer_mapped,
            delete_queue_framebuffers,
            delete_queue_image_views,
        }
    }

    pub fn destroy(&mut self, device: VkDevice) {
        unsafe {
            vkUnmapMemory(device, self.global_buffer.memory);
            self.global_buffer.destroy(device);

            vkUnmapMemory(device, self.object_buffer.memory);
            self.object_buffer.destroy(device);

            vkDestroySemaphore(device, self.image_available_semaphore, core::ptr::null());
            vkDestroySemaphore(device, self.render_finished_semaphore, core::ptr::null());
            vkDestroyFence(device, self.in_flight_fence, core::ptr::null());
            vkDestroyFence(device, self.present_fence, core::ptr::null());
            vkDestroyCommandPool(device, self.command_pool, core::ptr::null());
        }
    }
}
