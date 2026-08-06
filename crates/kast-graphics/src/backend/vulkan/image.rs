use vk_bindings::*;

use crate::backend::vulkan::{
    allocator::{Allocation, VulkanAllocator},
    command::VulkanCommandBuffer,
    deletion_queue::DeletionQueue,
    utils,
};

/// A Vulkan image (texture) with associated memory and image view.
///
/// Manages GPU image resources including memory allocation via the custom allocator,
/// image view creation, and provides methods for layout transitions and data upload.
/// Supports mipmap levels.
pub struct VulkanImage {
    pub(crate) handle: VkImage,
    pub(crate) view: VkImageView,
    pub(crate) format: VkFormat,
    pub(crate) bindless_slot: Option<u32>,
    pub allocation: Allocation,
    pub width: u32,
    pub height: u32,
    pub mip_levels: u32,
    device: VkDevice,
}

impl VulkanImage {
    /// Creates a new image with the specified parameters.
    ///
    /// # Arguments
    /// * `allocator` - The memory allocator to use
    /// * `device` - The Vulkan device
    /// * `width` - The width of the image
    /// * `height` - The height of the image
    /// * `format` - The image format
    /// * `tiling` - The image tiling mode
    /// * `usage` - The image usage flags
    /// * `memory_properties` - The memory property flags
    /// * `mip_levels` - The number of mip levels
    pub fn new(
        allocator: &mut VulkanAllocator,
        device: VkDevice,
        width: u32,
        height: u32,
        format: VkFormat,
        tiling: VkImageTiling,
        usage: VkImageUsageFlags,
        memory_properties: VkMemoryPropertyFlags,
        mip_levels: u32,
    ) -> Result<Self, String> {
        let create_info = VkImageCreateInfo {
            sType: VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            imageType: VK_IMAGE_TYPE_2D,
            format,
            extent: VkExtent3D {
                width,
                height,
                depth: 1,
            },
            mipLevels: mip_levels,
            arrayLayers: 1,
            samples: VK_SAMPLE_COUNT_1_BIT,
            tiling,
            usage,
            sharingMode: VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
            initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateImage(device, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create image: {}", result));
            }
        }

        // Get memory requirements
        let mut memory_requirements = VkMemoryRequirements::default();
        unsafe {
            vkGetImageMemoryRequirements(device, handle, &mut memory_requirements);
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
                    vkDestroyImage(device, handle, core::ptr::null());
                }
                e
            })?;

        // Bind image to memory
        unsafe {
            let result = vkBindImageMemory(device, handle, allocation.memory, allocation.offset);
            if result != VK_SUCCESS {
                allocator.free(&allocation);
                vkDestroyImage(device, handle, core::ptr::null());
                return Err(format!("Failed to bind image memory: {}", result));
            }
        }

        // Create image view
        let view = utils::create_image_view(
            device,
            handle,
            format,
            VK_IMAGE_ASPECT_COLOR_BIT,
            mip_levels,
        )
        .map_err(|e| {
            unsafe {
                vkDestroyImage(device, handle, core::ptr::null());
            }
            allocator.free(&allocation);
            e
        })?;

        Ok(Self {
            handle,
            view,
            allocation,
            width,
            height,
            format,
            mip_levels,
            bindless_slot: None,
            device: device,
        })
    }

    /// Transitions the image layout using an image memory barrier.
    ///
    /// # Arguments
    /// * `command_buffer` - The command buffer to record the barrier into
    /// * `old_layout` - The current layout of the image
    /// * `new_layout` - The desired layout of the image
    pub fn transition_layout(
        &self,
        command_buffer: &VulkanCommandBuffer,
        old_layout: VkImageLayout,
        new_layout: VkImageLayout,
    ) {
        let (src_access_mask, dst_access_mask, src_stage, dst_stage) =
            Self::get_transition_params(old_layout, new_layout);

        let barrier = VkImageMemoryBarrier {
            sType: VK_STRUCTURE_TYPE_IMAGE_MEMORY_BARRIER,
            pNext: core::ptr::null(),
            srcAccessMask: src_access_mask,
            dstAccessMask: dst_access_mask,
            oldLayout: old_layout,
            newLayout: new_layout,
            srcQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED as u32,
            dstQueueFamilyIndex: VK_QUEUE_FAMILY_IGNORED as u32,
            image: self.handle,
            subresourceRange: VkImageSubresourceRange {
                aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                baseMipLevel: 0,
                levelCount: self.mip_levels,
                baseArrayLayer: 0,
                layerCount: 1,
            },
        };

        command_buffer.pipeline_barrier(src_stage, dst_stage, 0, &[], &[], &[barrier]);
    }

    /// Helper function to determine transition parameters.
    fn get_transition_params(
        old_layout: VkImageLayout,
        new_layout: VkImageLayout,
    ) -> (
        VkAccessFlags,
        VkAccessFlags,
        VkPipelineStageFlags,
        VkPipelineStageFlags,
    ) {
        match (old_layout, new_layout) {
            // Undefined -> Transfer Dst: Preparing for data upload
            (VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL) => (
                0,
                VK_ACCESS_TRANSFER_WRITE_BIT,
                VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
            ),
            // Transfer Dst -> Shader Read: After upload, prepare for shader access
            (VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL) => (
                VK_ACCESS_TRANSFER_WRITE_BIT,
                VK_ACCESS_SHADER_READ_BIT,
                VK_PIPELINE_STAGE_TRANSFER_BIT,
                VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
            ),
            // Undefined -> Shader Read: Direct transition for pre-initialized images
            (VK_IMAGE_LAYOUT_UNDEFINED, VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL) => (
                0,
                VK_ACCESS_SHADER_READ_BIT,
                VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                VK_PIPELINE_STAGE_FRAGMENT_SHADER_BIT,
            ),
            // Color Attachment -> Present: After rendering, prepare for presentation
            (VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL, VK_IMAGE_LAYOUT_PRESENT_SRC_KHR) => (
                VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
                0,
                VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
            ),
            // Generic fallback
            _ => (
                0,
                0,
                VK_PIPELINE_STAGE_TOP_OF_PIPE_BIT,
                VK_PIPELINE_STAGE_BOTTOM_OF_PIPE_BIT,
            ),
        }
    }

    /// Copies data from a buffer to this image.
    ///
    /// # Arguments
    /// * `command_buffer` - The command buffer to record the copy into
    /// * `buffer` - The source buffer
    /// * `width` - The width of the region to copy
    /// * `height` - The height of the region to copy
    pub fn copy_from_buffer(
        &self,
        command_buffer: &VulkanCommandBuffer,
        buffer: VkBuffer,
        width: u32,
        height: u32,
    ) {
        let region = VkBufferImageCopy {
            bufferOffset: 0,
            bufferRowLength: 0,
            bufferImageHeight: 0,
            imageSubresource: VkImageSubresourceLayers {
                aspectMask: VK_IMAGE_ASPECT_COLOR_BIT,
                mipLevel: 0,
                baseArrayLayer: 0,
                layerCount: 1,
            },
            imageOffset: VkOffset3D { x: 0, y: 0, z: 0 },
            imageExtent: VkExtent3D {
                width,
                height,
                depth: 1,
            },
        };

        command_buffer.copy_buffer_to_image(
            buffer,
            self.handle,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    /// Destroys the image by scheduling cleanup via the deletion queue.
    ///
    /// This is the recommended way to destroy images when using multiple frames in flight.
    /// The Vulkan handles will be destroyed when the current frame completes, and the
    /// memory allocation will be freed automatically.
    ///
    /// # Arguments
    /// * `deletion_queue` - The deletion queue to schedule cleanup with
    pub fn destroy(self, deletion_queue: &mut DeletionQueue) {
        let handle = self.handle;
        let view = self.view;
        let allocation = self.allocation.clone();
        let device = self.device;
        core::mem::forget(self);

        deletion_queue.push_with_allocation(allocation, move || unsafe {
            vkDestroyImageView(device, view, core::ptr::null());
            vkDestroyImage(device, handle, core::ptr::null());
        });
    }

    /// Destroys the image immediately.
    /// Only safe when the GPU is known to be idle or the image was never used.
    ///
    /// # Arguments
    /// * `allocator` - The Vulkan allocator to free the memory allocation with
    pub fn destroy_immediate(self, allocator: &mut VulkanAllocator) {
        let handle = self.handle;
        let view = self.view;
        let allocation = self.allocation.clone();
        let device = self.device;
        core::mem::forget(self);

        unsafe {
            vkDestroyImageView(device, view, core::ptr::null());
            vkDestroyImage(device, handle, core::ptr::null());
        }
        allocator.free(&allocation);
    }
}

impl Drop for VulkanImage {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            panic!(
                "VulkanImage must not be dropped directly. Use destroy() with a deletion queue."
            );
        }
    }
}
