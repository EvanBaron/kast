use vk_bindings::*;

/// Represents a queue family and queue index for Vulkan queue operations.
///
/// Contains both the family index (which type of operations the queue supports)
/// and the queue index within that family.
#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    pub family_index: u32,
    pub queue_index: u32,
}

impl QueueFamily {
    /// Creates a new QueueFamily with the specified family and queue indices.
    ///
    /// # Arguments
    /// * `family_index` - The queue family index.
    /// * `queue_index` - The queue index within the family.
    pub fn new(family_index: u32, queue_index: u32) -> Self {
        Self {
            family_index,
            queue_index,
        }
    }
}

/// Checks if the physical device supports the required swapchain extension.
///
/// # Arguments
/// * `physical_device` - The physical device to check.
///
/// # Returns
/// `true` if VK_KHR_swapchain extension is supported, `false` otherwise.
pub fn check_device_extension_support(physical_device: VkPhysicalDevice) -> bool {
    let mut count = 0;
    unsafe {
        vkEnumerateDeviceExtensionProperties(
            physical_device,
            core::ptr::null(),
            &mut count,
            core::ptr::null_mut(),
        );
    }

    let mut device_extensions = vec![VkExtensionProperties::default(); count as usize];
    unsafe {
        vkEnumerateDeviceExtensionProperties(
            physical_device,
            core::ptr::null(),
            &mut count,
            device_extensions.as_mut_ptr(),
        );
    }

    let extension_name =
        unsafe { core::ffi::CStr::from_bytes_with_nul_unchecked(VK_KHR_SWAPCHAIN_EXTENSION_NAME) };

    device_extensions.iter().any(|extension_properties| {
        let current_extension_name =
            unsafe { core::ffi::CStr::from_ptr(extension_properties.extensionName.as_ptr()) };

        current_extension_name == extension_name
    })
}

/// Finds suitable queue families for graphics and presentation operations.
///
/// Searches for queue families that support graphics operations and surface presentation.
/// The graphics and present queue families may be the same or different depending on the device.
///
/// # Arguments
/// * `physical_device` - The physical device to query.
/// * `surface` - The surface to check presentation support against.
///
/// # Returns
/// A tuple of (graphics queue family, present queue family) or an error if suitable families are not found.
pub fn find_queue_families(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> Result<(QueueFamily, QueueFamily), &'static str> {
    let mut count = 0;
    unsafe {
        vkGetPhysicalDeviceQueueFamilyProperties(
            physical_device,
            &mut count,
            core::ptr::null_mut(),
        );
    }

    let mut properties = vec![VkQueueFamilyProperties::default(); count as usize];
    unsafe {
        vkGetPhysicalDeviceQueueFamilyProperties(
            physical_device,
            &mut count,
            properties.as_mut_ptr(),
        );
    }

    let mut graphics_family = None;
    let mut present_family = None;

    for (i, property) in properties.iter().enumerate() {
        let index = i as u32;

        // Check Graphics support
        if (property.queueFlags & VK_QUEUE_GRAPHICS_BIT) != 0 {
            graphics_family = Some(index);
        }

        // Check Present support
        let mut present_support = VK_FALSE;
        unsafe {
            vkGetPhysicalDeviceSurfaceSupportKHR(
                physical_device,
                index,
                surface,
                &mut present_support,
            );
        }

        if present_support == VK_TRUE {
            present_family = Some(index);
        }

        if graphics_family.is_some() && present_family.is_some() {
            break;
        }
    }

    if let (Some(g), Some(p)) = (graphics_family, present_family) {
        Ok((
            QueueFamily {
                family_index: g,
                queue_index: 0,
            },
            QueueFamily {
                family_index: p,
                queue_index: 0,
            },
        ))
    } else {
        Err("Could not find suitable queue families")
    }
}

/// Creates an image view for the given image.
///
/// Image views describe how to access images and which portions to access.
///
/// # Arguments
/// * `device` - The Vulkan device.
/// * `image` - The image to create a view for.
/// * `format` - The format of the image.
/// * `aspect_flags` - The aspect mask.
/// * `mip_levels` - The number of mip levels.
///
/// # Returns
/// A VkImageView handle or an error if creation fails.
pub fn create_image_view(
    device: VkDevice,
    image: VkImage,
    format: VkFormat,
    aspect_flags: VkImageAspectFlags,
    mip_levels: u32,
) -> Result<VkImageView, String> {
    let create_info = VkImageViewCreateInfo {
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
            levelCount: mip_levels,
            baseArrayLayer: 0,
            layerCount: 1,
        },
    };

    let mut view = core::ptr::null_mut();
    unsafe {
        let result = vkCreateImageView(device, &create_info, core::ptr::null(), &mut view);
        if result != VK_SUCCESS {
            return Err(format!("Failed to create image view: {}", result));
        }
    }
    Ok(view)
}

/// Executes a command buffer immediately and waits for it to complete.
///
/// This is a utility function for one-time command buffer operations like
/// buffer uploads and image layout transitions during initialization.
///
/// # Arguments
/// * `device` - The Vulkan device.
/// * `command_pool` - The command pool to allocate the command buffer from.
/// * `queue` - The queue to submit the command buffer to.
/// * `function` - A closure that records commands into the command buffer.
pub fn immediate_submit<F>(
    device: VkDevice,
    command_pool: VkCommandPool,
    queue: VkQueue,
    function: F,
) -> Result<(), String>
where
    F: FnOnce(VkCommandBuffer),
{
    let allocate_info = VkCommandBufferAllocateInfo {
        sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
        pNext: core::ptr::null(),
        commandPool: command_pool,
        level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
        commandBufferCount: 1,
    };

    let mut command_buffer = core::ptr::null_mut();
    unsafe {
        let result = vkAllocateCommandBuffers(device, &allocate_info, &mut command_buffer);
        if result != VK_SUCCESS {
            return Err(format!(
                "immediate_submit: failed to allocate command buffer: {}",
                result
            ));
        }

        let begin_info = VkCommandBufferBeginInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT,
            pInheritanceInfo: core::ptr::null(),
        };

        let result = vkBeginCommandBuffer(command_buffer, &begin_info);
        if result != VK_SUCCESS {
            vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
            return Err(format!(
                "immediate_submit: failed to begin command buffer: {}",
                result
            ));
        }

        // Execute user function to record commands
        function(command_buffer);

        let result = vkEndCommandBuffer(command_buffer);
        if result != VK_SUCCESS {
            vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
            return Err(format!(
                "immediate_submit: failed to end command buffer: {}",
                result
            ));
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

        let result = vkQueueSubmit(queue, 1, &submit_info, core::ptr::null_mut());
        if result != VK_SUCCESS {
            vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);
            return Err(format!(
                "immediate_submit: failed to submit command buffer: {}",
                result
            ));
        }

        let result = vkQueueWaitIdle(queue);
        vkFreeCommandBuffers(device, command_pool, 1, &command_buffer);

        if result != VK_SUCCESS {
            return Err(format!("immediate_submit: queue wait failed: {}", result));
        }
    }

    Ok(())
}

pub fn find_memory_type(
    physical_device: VkPhysicalDevice,
    type_filter: u32,
    properties: VkMemoryPropertyFlags,
) -> Result<u32, String> {
    let mut memory_properties = VkPhysicalDeviceMemoryProperties::default();
    unsafe {
        vkGetPhysicalDeviceMemoryProperties(physical_device, &mut memory_properties);
    }

    for i in 0..memory_properties.memoryTypeCount {
        if (type_filter & (1 << i)) != 0
            && (memory_properties.memoryTypes[i as usize].propertyFlags & properties) == properties
        {
            return Ok(i);
        }
    }

    Err("Failed to find suitable memory type!".to_string())
}
