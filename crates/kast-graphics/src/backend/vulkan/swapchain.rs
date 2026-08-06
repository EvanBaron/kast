use vk_bindings::*;

use crate::backend::vulkan::{device::VulkanDevice, utils};

/// A Vulkan swapchain for presenting rendered images to the screen.
///
/// Manages a chain of images that can be rendered to and presented to the window surface.
/// Automatically creates image views and can manage framebuffers for each swapchain image.
pub struct VulkanSwapchain {
    pub(crate) handle: VkSwapchainKHR,
    pub(crate) format: VkSurfaceFormatKHR,
    pub(crate) extent: VkExtent2D,
    pub(crate) images: Vec<VkImage>,
    pub(crate) image_views: Vec<VkImageView>,
    pub(crate) framebuffers: Vec<VkFramebuffer>,
    device: VkDevice,
}

impl VulkanSwapchain {
    /// Creates a new swapchain for the given surface and window dimensions.
    ///
    /// Automatically selects appropriate format, present mode, and extent based on
    /// surface capabilities. Creates image views for all swapchain images.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `surface` - The window surface to present to.
    /// * `window_width` - The width of the window in pixels.
    /// * `window_height` - The height of the window in pixels.
    /// * `old_swapchain` - Optional old swapchain for recreation.
    ///
    /// # Returns
    /// A new VulkanSwapchain or an error if creation fails.
    pub fn new(
        device: &VulkanDevice,
        surface: VkSurfaceKHR,
        window_width: u32,
        window_height: u32,
        old_swapchain: Option<&VulkanSwapchain>,
    ) -> Result<Self, String> {
        let swapchain_support = query_swapchain_support(device.physical_device, surface);

        let surface_format = choose_swapchain_surface_format(&swapchain_support.formats);
        let present_mode = choose_swapchain_present_mode(&swapchain_support.present_modes);
        let extent =
            choose_swapchain_extent(&swapchain_support.capabilities, window_width, window_height);

        let mut image_count = swapchain_support.capabilities.minImageCount + 1;
        if swapchain_support.capabilities.maxImageCount > 0
            && image_count > swapchain_support.capabilities.maxImageCount
        {
            image_count = swapchain_support.capabilities.maxImageCount;
        }

        let mut create_info = VkSwapchainCreateInfoKHR {
            sType: VK_STRUCTURE_TYPE_SWAPCHAIN_CREATE_INFO_KHR,
            pNext: core::ptr::null(),
            flags: 0,
            surface,
            minImageCount: image_count,
            imageFormat: surface_format.format,
            imageColorSpace: surface_format.colorSpace,
            imageExtent: extent,
            imageArrayLayers: 1,
            imageUsage: VK_IMAGE_USAGE_COLOR_ATTACHMENT_BIT,
            imageSharingMode: VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
            preTransform: swapchain_support.capabilities.currentTransform,
            compositeAlpha: VK_COMPOSITE_ALPHA_OPAQUE_BIT_KHR,
            presentMode: present_mode,
            clipped: VK_TRUE,
            oldSwapchain: old_swapchain
                .map(|s| s.handle)
                .unwrap_or(core::ptr::null_mut()),
        };

        // Handle Queue Families (if graphics != present)
        let indices = [
            device.graphics_family.family_index,
            device.present_family.family_index,
        ];
        if device.graphics_family.family_index != device.present_family.family_index {
            create_info.imageSharingMode = VK_SHARING_MODE_CONCURRENT;
            create_info.queueFamilyIndexCount = 2;
            create_info.pQueueFamilyIndices = indices.as_ptr();
        }

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreateSwapchainKHR(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create swapchain: {}", result));
            }
        }

        let mut swapchain_image_count = 0;
        unsafe {
            vkGetSwapchainImagesKHR(
                device.handle,
                handle,
                &mut swapchain_image_count,
                core::ptr::null_mut(),
            );
        }

        let mut images = vec![core::ptr::null_mut(); swapchain_image_count as usize];
        unsafe {
            vkGetSwapchainImagesKHR(
                device.handle,
                handle,
                &mut swapchain_image_count,
                images.as_mut_ptr(),
            );
        }

        let mut image_views = Vec::with_capacity(images.len());
        for &image in &images {
            let view = utils::create_image_view(
                device.handle,
                image,
                surface_format.format,
                VK_IMAGE_ASPECT_COLOR_BIT,
                1,
            )?;

            image_views.push(view);
        }

        Ok(Self {
            handle,
            format: surface_format,
            extent,
            images,
            image_views,
            framebuffers: Vec::new(),
            device: device.handle,
        })
    }

    /// Creates framebuffers for all swapchain images with the given render pass.
    ///
    /// Automatically destroys any existing framebuffers before creating new ones.
    /// Each framebuffer corresponds to one swapchain image view.
    ///
    /// # Arguments
    /// * `render_pass` - The render pass that the framebuffers will be compatible with.
    ///
    /// # Returns
    /// Ok(()) on success, or an error string if framebuffer creation fails.
    pub fn create_framebuffers(&mut self, render_pass: VkRenderPass) -> Result<(), String> {
        // Clear old framebuffers
        unsafe {
            for &framebuffer in &self.framebuffers {
                vkDestroyFramebuffer(self.device, framebuffer, core::ptr::null());
            }
        }
        self.framebuffers.clear();

        for &view in &self.image_views {
            let attachments = [view];

            let create_info = VkFramebufferCreateInfo {
                sType: VK_STRUCTURE_TYPE_FRAMEBUFFER_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                renderPass: render_pass,
                attachmentCount: 1,
                pAttachments: attachments.as_ptr(),
                width: self.extent.width,
                height: self.extent.height,
                layers: 1,
            };

            let mut framebuffer = core::ptr::null_mut();
            unsafe {
                let result = vkCreateFramebuffer(
                    self.device,
                    &create_info,
                    core::ptr::null(),
                    &mut framebuffer,
                );
                if result != VK_SUCCESS {
                    return Err(format!("Failed to create framebuffer: {}", result));
                }
            }

            self.framebuffers.push(framebuffer);
        }

        Ok(())
    }
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        unsafe {
            for &framebuffer in &self.framebuffers {
                vkDestroyFramebuffer(self.device, framebuffer, core::ptr::null());
            }

            for &view in &self.image_views {
                vkDestroyImageView(self.device, view, core::ptr::null());
            }

            vkDestroySwapchainKHR(self.device, self.handle, core::ptr::null());
        }
    }
}

struct SwapChainSupportDetails {
    capabilities: VkSurfaceCapabilitiesKHR,
    formats: Vec<VkSurfaceFormatKHR>,
    present_modes: Vec<VkPresentModeKHR>,
}

fn query_swapchain_support(
    physical_device: VkPhysicalDevice,
    surface: VkSurfaceKHR,
) -> SwapChainSupportDetails {
    unsafe {
        let mut capabilities = VkSurfaceCapabilitiesKHR::default();
        vkGetPhysicalDeviceSurfaceCapabilitiesKHR(physical_device, surface, &mut capabilities);

        let mut format_count = 0;
        vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface,
            &mut format_count,
            core::ptr::null_mut(),
        );
        let mut formats = vec![VkSurfaceFormatKHR::default(); format_count as usize];
        vkGetPhysicalDeviceSurfaceFormatsKHR(
            physical_device,
            surface,
            &mut format_count,
            formats.as_mut_ptr(),
        );

        let mut mode_count = 0;
        vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface,
            &mut mode_count,
            core::ptr::null_mut(),
        );
        let mut present_modes = vec![VkPresentModeKHR::default(); mode_count as usize];
        vkGetPhysicalDeviceSurfacePresentModesKHR(
            physical_device,
            surface,
            &mut mode_count,
            present_modes.as_mut_ptr(),
        );

        SwapChainSupportDetails {
            capabilities,
            formats,
            present_modes,
        }
    }
}

fn choose_swapchain_surface_format(available_formats: &[VkSurfaceFormatKHR]) -> VkSurfaceFormatKHR {
    for format in available_formats {
        if format.format == VK_FORMAT_B8G8R8A8_SRGB
            && format.colorSpace == VK_COLOR_SPACE_SRGB_NONLINEAR_KHR
        {
            return *format;
        }
    }

    available_formats[0]
}

fn choose_swapchain_present_mode(available_present_modes: &[VkPresentModeKHR]) -> VkPresentModeKHR {
    for &mode in available_present_modes {
        if mode == VK_PRESENT_MODE_MAILBOX_KHR {
            return mode;
        }
    }

    VK_PRESENT_MODE_FIFO_KHR
}

fn choose_swapchain_extent(
    capabilities: &VkSurfaceCapabilitiesKHR,
    window_width: u32,
    window_height: u32,
) -> VkExtent2D {
    if capabilities.currentExtent.width != u32::MAX {
        return capabilities.currentExtent;
    }

    let mut extent = VkExtent2D {
        width: window_width,
        height: window_height,
    };

    extent.width = extent.width.clamp(
        capabilities.minImageExtent.width,
        capabilities.maxImageExtent.width,
    );
    extent.height = extent.height.clamp(
        capabilities.minImageExtent.height,
        capabilities.maxImageExtent.height,
    );

    extent
}
