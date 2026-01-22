use crate::graphics::instance::QueueFamily;
use vk_bindings::*;
use winit::window::Window;

pub struct Swapchain {
    pub handle: VkSwapchainKHR,
    pub image_count: u32,
    pub image_views: Vec<VkImageView>,
    pub surface_format: VkSurfaceFormatKHR,
    pub extent: VkExtent2D,
    pub framebuffers: Vec<VkFramebuffer>,
    device: VkDevice,
}

impl Swapchain {
    /// Creates a swapchain for the given surface.
    ///
    /// # Arguments
    /// * `physical_device` - The physical device to create the swapchain for.
    /// * `surface` - The surface to create the swapchain for.
    /// * `graphics_queue_family` - The graphics queue family to use for the swapchain.
    /// * `present_queue_family` - The present queue family to use for the swapchain.
    pub fn new(
        physical_device: VkPhysicalDevice,
        device: VkDevice,
        surface: VkSurfaceKHR,
        graphics_queue_family: QueueFamily,
        present_queue_family: QueueFamily,
        window: &Window,
        old_swapchain: VkSwapchainKHR,
        surface_format: Option<VkSurfaceFormatKHR>,
    ) -> Self {
        let surface_format = match surface_format {
            Some(surface_format) => surface_format,
            None => Self::choose_surface_format(physical_device, surface),
        };

        let mut present_mode_count: u32 = 0;
        unsafe {
            vkGetPhysicalDeviceSurfacePresentModesKHR(
                physical_device,
                surface,
                &mut present_mode_count,
                core::ptr::null_mut(),
            )
        };

        let mut present_modes = vec![VkPresentModeKHR::default(); present_mode_count as usize];
        unsafe {
            vkGetPhysicalDeviceSurfacePresentModesKHR(
                physical_device,
                surface,
                &mut present_mode_count,
                present_modes.as_mut_ptr(),
            )
        };

        // Select the present mode VK_PRESENT_MODE_MAILBOX_KHR first, then VK_PRESENT_MODE_FIFO_KHR as a fallback.
        let present_mode = if present_modes.contains(&VK_PRESENT_MODE_MAILBOX_KHR) {
            VK_PRESENT_MODE_MAILBOX_KHR
        } else {
            VK_PRESENT_MODE_FIFO_KHR
        };

        let empty_array = [];
        let queue_family_array = [
            graphics_queue_family.family_index,
            present_queue_family.family_index,
        ];

        let image_sharing_mode;
        let queue_families;

        // Exclusive sharing mode if the same queue family is used for graphics and presentation
        if graphics_queue_family.family_index == present_queue_family.family_index {
            image_sharing_mode = VK_SHARING_MODE_EXCLUSIVE;
            queue_families = &empty_array[..];
        // Concurrent sharing mode if different queue families are used for graphics and presentation
        } else {
            image_sharing_mode = VK_SHARING_MODE_CONCURRENT;
            queue_families = &queue_family_array[..];
        };

        let mut surface_capabilities = VkSurfaceCapabilitiesKHR::default();
        unsafe {
            vkGetPhysicalDeviceSurfaceCapabilitiesKHR(
                physical_device,
                surface,
                &mut surface_capabilities,
            );
        }

        let mut image_count = surface_capabilities.minImageCount + 1;
        if surface_capabilities.maxImageCount > 0
            && image_count > surface_capabilities.maxImageCount
        {
            image_count = surface_capabilities.maxImageCount;
        }

        // Determine the swap extent (resolution of the swap chain images).
        // If the surface size is undefined (u32::MAX), we clamp the window size to the
        // min/max extent supported by the surface.
        let extent = if surface_capabilities.currentExtent.width != u32::MAX {
            surface_capabilities.currentExtent
        } else {
            let min_width = surface_capabilities.minImageExtent.width;
            let max_width = surface_capabilities.maxImageExtent.width;

            let min_height = surface_capabilities.minImageExtent.height;
            let max_height = surface_capabilities.maxImageExtent.height;

            let width = window.inner_size().width;
            let height = window.inner_size().height;

            VkExtent2D {
                width: min_width.max(max_width.min(width)),
                height: min_height.max(max_height.min(height)),
            }
        };

        let swapchain_create_info = VkSwapchainCreateInfoKHR {
            sType: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            flags: 0x0,
            pNext: core::ptr::null(),
            surface: surface,
            minImageCount: image_count,
            imageFormat: surface_format.format,
            imageColorSpace: surface_format.colorSpace,
            imageExtent: extent,
            imageArrayLayers: 1,
            imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT as VkImageUsageFlags,
            imageSharingMode: image_sharing_mode,
            queueFamilyIndexCount: queue_families.len() as u32,
            pQueueFamilyIndices: queue_families.as_ptr(),
            preTransform: surface_capabilities.currentTransform,
            compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode: present_mode,
            clipped: VK_TRUE,
            oldSwapchain: old_swapchain,
        };

        println!("Creating swapchain.");
        let mut swapchain = core::ptr::null_mut();
        let result = unsafe {
            vkCreateSwapchainKHR(
                device,
                &swapchain_create_info,
                core::ptr::null_mut(),
                &mut swapchain,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create swapchain!");
        }

        if old_swapchain != core::ptr::null_mut() {
            println!("Deleting old swapchain.");
            unsafe {
                vkDestroySwapchainKHR(device, old_swapchain, core::ptr::null_mut());
            }
        }

        let mut image_count: u32 = 0;
        let result = unsafe {
            vkGetSwapchainImagesKHR(device, swapchain, &mut image_count, core::ptr::null_mut())
        };

        if result != VK_SUCCESS {
            panic!("Failed to get swapchain images. Error: {:?}.", result);
        }

        let mut images = vec![core::ptr::null_mut(); image_count as usize];
        let result = unsafe {
            vkGetSwapchainImagesKHR(device, swapchain, &mut image_count, images.as_mut_ptr())
        };

        if result != VK_SUCCESS {
            panic!("Failed to get swapchain images. Error: {:?}.", result);
        }

        let mut image_views = Vec::with_capacity(image_count as usize);
        for (i, swapchain_image) in images.iter().enumerate() {
            let img_view_create_info = VkImageViewCreateInfo {
                sType: VK_STRUCTURE_TYPE_IMAGE_VIEW_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                image: *swapchain_image,
                viewType: VK_IMAGE_VIEW_TYPE_2D,
                format: surface_format.format,
                components: VkComponentMapping {
                    r: VK_COMPONENT_SWIZZLE_IDENTITY,
                    g: VK_COMPONENT_SWIZZLE_IDENTITY,
                    b: VK_COMPONENT_SWIZZLE_IDENTITY,
                    a: VK_COMPONENT_SWIZZLE_IDENTITY,
                },
                subresourceRange: VkImageSubresourceRange {
                    aspectMask: VK_IMAGE_ASPECT_COLOR_BIT as VkImageAspectFlags,
                    baseMipLevel: 0,
                    levelCount: 1,
                    baseArrayLayer: 0,
                    layerCount: 1,
                },
            };

            println!("Creating framebuffer image view.");
            let mut image_view = core::ptr::null_mut();
            let result = unsafe {
                vkCreateImageView(
                    device,
                    &img_view_create_info,
                    core::ptr::null_mut(),
                    &mut image_view,
                )
            };

            if result != VK_SUCCESS {
                panic!(
                    "Failed to create framebuffer image view {:?}. Error: {:?}.",
                    i, result
                );
            }

            image_views.push(image_view);
        }

        Self {
            handle: swapchain,
            device,
            image_count,
            image_views,
            surface_format,
            extent,
            framebuffers: Vec::new(),
        }
    }

    /// Creates framebuffers for each image view in the swapchain.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `render_pass` - The Vulkan render pass.
    pub fn create_framebuffers(&mut self, device: VkDevice, render_pass: VkRenderPass) {
        let mut framebuffers = Vec::with_capacity(self.image_count as usize);
        for (i, swapchain_image_view) in self.image_views.iter().enumerate() {
            let attachments: [VkImageView; 1] = [*swapchain_image_view];

            let create_info = VkFramebufferCreateInfo {
                sType: VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                renderPass: render_pass,
                attachmentCount: attachments.len() as u32,
                pAttachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
            };

            println!("Creating framebuffer.");
            let mut new_framebuffer = core::ptr::null_mut();
            let result = unsafe {
                vkCreateFramebuffer(
                    device,
                    &create_info,
                    core::ptr::null_mut(),
                    &mut new_framebuffer,
                )
            };

            if result != VK_SUCCESS {
                panic!("Failed to create framebuffer {:?}. Error: {:?}.", i, result);
            }

            framebuffers.push(new_framebuffer);
        }

        self.framebuffers = framebuffers;
    }

    /// Selects the surface format for the swapchain that supports the correct color space.
    ///
    /// # Arguments
    /// * `physical_device` - The physical device to query.
    /// * `surface` - The surface to query.
    fn choose_surface_format(
        physical_device: VkPhysicalDevice,
        surface: VkSurfaceKHR,
    ) -> VkSurfaceFormatKHR {
        let mut surface_format_count: u32 = 0;
        unsafe {
            vkGetPhysicalDeviceSurfaceFormatsKHR(
                physical_device,
                surface,
                &mut surface_format_count,
                core::ptr::null_mut(),
            )
        };

        let mut surface_formats =
            vec![VkSurfaceFormatKHR::default(); surface_format_count as usize];
        unsafe {
            vkGetPhysicalDeviceSurfaceFormatsKHR(
                physical_device,
                surface,
                &mut surface_format_count,
                surface_formats.as_mut_ptr(),
            )
        };

        // Select the surface format:
        // We look for a format that supports the B8G8R8A8_UNORM format and SRGB color space.
        // SRGB is preferred for correct color rendering (gamma correction).
        let format = surface_formats.iter().find(|format| {
            let mut format_properties = VkFormatProperties::default();
            unsafe {
                vkGetPhysicalDeviceFormatProperties(
                    physical_device,
                    format.format,
                    &mut format_properties,
                )
            };

            (format_properties.optimalTilingFeatures
                & VK_FORMAT_FEATURE_COLOR_ATTACHMENT_BIT as VkFormatFeatureFlags)
                != 0
                && format.format == VK_FORMAT_B8G8R8A8_UNORM
                && format.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
        });

        match format {
            Some(format) => *format,
            None => {
                println!("Preferred surface format not found. Falling back to first available.");
                surface_formats[0]
            }
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            for &framebuffer in self.framebuffers.iter() {
                vkDestroyFramebuffer(self.device, framebuffer, core::ptr::null_mut());
            }
            self.framebuffers.clear();

            for &image_view in self.image_views.iter() {
                vkDestroyImageView(self.device, image_view, core::ptr::null_mut());
            }
            self.image_views.clear();
        }
        unsafe {
            vkDestroySwapchainKHR(self.device, self.handle, core::ptr::null_mut());
        }
    }
}
