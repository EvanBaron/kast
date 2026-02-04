use vk_bindings::*;

/// Finds a suitable memory type index for the given requirements and properties.
///
/// # Arguments
/// * `physical_device` - The Vulkan physical device.
/// * `type_filter` - Bit field indicating suitable memory types (from VkMemoryRequirements).
/// * `properties` - The required memory property flags (e.g., HOST_VISIBLE, DEVICE_LOCAL).
///
/// # Returns
/// The index of the suitable memory type. Panics if no suitable memory type is found.
pub fn find_memory_type(
    physical_device: VkPhysicalDevice,
    type_filter: u32,
    properties: VkMemoryPropertyFlags,
) -> u32 {
    let mut memory_properties = VkPhysicalDeviceMemoryProperties::default();
    unsafe {
        vkGetPhysicalDeviceMemoryProperties(physical_device, &mut memory_properties);
    }

    for i in 0..memory_properties.memoryTypeCount {
        if (type_filter & (1 << i)) != 0
            && (memory_properties.memoryTypes[i as usize].propertyFlags & properties) == properties
        {
            return i;
        }
    }

    panic!("Failed to find suitable memory type!");
}

/// Creates a new image view for the given image.
///
/// # Arguments
/// * `device` - The Vulkan device.
/// * `image` - The image to create the view for.
/// * `format` - The format of the image.
/// * `aspect_flags` - The aspect mask for the image view (e.g. VK_IMAGE_ASPECT_COLOR_BIT).
pub fn create_image_view(
    device: VkDevice,
    image: VkImage,
    format: VkFormat,
    aspect_flags: VkImageAspectFlags,
) -> VkImageView {
    let image_view_create_info = VkImageViewCreateInfo {
        sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
        pNext: core::ptr::null(),
        flags: 0,
        image,
        viewType: VK_IMAGE_VIEW_TYPE_2D,
        format,
        components: VkComponentMapping {
            r: VK_COMPONENT_SWIZZLE_IDENTITY,
            g: VK_COMPONENT_SWIZZLE_IDENTITY,
            b: VK_COMPONENT_SWIZZLE_IDENTITY,
            a: VK_COMPONENT_SWIZZLE_IDENTITY,
        },
        subresourceRange: VkImageSubresourceRange {
            aspectMask: aspect_flags,
            baseMipLevel: 0,
            levelCount: 1,
            baseArrayLayer: 0,
            layerCount: 1,
        },
    };

    let mut image_view = core::ptr::null_mut();
    unsafe {
        let result = vkCreateImageView(
            device,
            &image_view_create_info,
            core::ptr::null_mut(),
            &mut image_view,
        );

        if result != VK_SUCCESS {
            panic!("Failed to create image view. Error: {}", result);
        }
    }

    image_view
}

/// Submits a command buffer to the queue and waits for it to finish.
///
/// # Arguments
/// * `device` - The Vulkan device.
/// * `command_pool` - The command pool to allocate the command buffer from.
/// * `queue` - The queue to submit the command buffer to.
/// * `function` - The function to execute with the command buffer.
pub fn immediate_submit<F>(
    device: VkDevice,
    command_pool: VkCommandPool,
    queue: VkQueue,
    function: F,
) where
    F: FnOnce(VkCommandBuffer),
{
    let command_buffer_allocate_info = VkCommandBufferAllocateInfo {
        sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: core::ptr::null(),
        commandPool: command_pool,
        level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: 1,
    };

    let mut command_buffer = core::ptr::null_mut();
    unsafe {
        vkAllocateCommandBuffers(device, &command_buffer_allocate_info, &mut command_buffer);

        let command_buffer_begin_info = VkCommandBufferBeginInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
            pInheritanceInfo: core::ptr::null(),
        };

        vkBeginCommandBuffer(command_buffer, &command_buffer_begin_info);
        function(command_buffer);
        vkEndCommandBuffer(command_buffer);

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

        vkQueueSubmit(queue, 1, &submit_info, core::ptr::null_mut());
        vkQueueWaitIdle(queue);

        vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
    };
}
