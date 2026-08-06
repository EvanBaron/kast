use vk_bindings::*;

use crate::backend::vulkan::device::VulkanDevice;

/// A Vulkan render pass defining framebuffer attachments and rendering operations.
///
/// Render passes describe the structure of attachments (color, depth, stencil) and their
/// usage during rendering, including load/store operations and layout transitions.
/// Automatically destroyed when dropped.
pub struct VulkanRenderPass {
    pub(crate) handle: VkRenderPass,
    device: VkDevice,
}

impl VulkanRenderPass {
    /// Creates a render pass with a color attachment and optional depth attachment.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `color_format` - The format of the color attachment.
    /// * `depth_format` - Optional depth attachment format. If None, no depth attachment is created.
    /// * `final_layout` - The layout to transition the color attachment to after the render pass.
    pub fn new(
        device: &VulkanDevice,
        color_format: VkFormat,
        depth_format: Option<VkFormat>,
        final_layout: VkImageLayout,
    ) -> Result<Self, String> {
        let color_attachment_description = VkAttachmentDescription {
            flags: 0,
            format: color_format,
            samples: VK_SAMPLE_COUNT_1_BIT,
            loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
            storeOp: VK_ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
            finalLayout: final_layout,
        };

        let color_attachment_reference = VkAttachmentReference {
            attachment: 0,
            layout: VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        let (attachments, depth_attachment_reference, dependency) =
            if let Some(depth_format) = depth_format {
                let depth_attachment_description = VkAttachmentDescription {
                    flags: 0,
                    format: depth_format,
                    samples: VK_SAMPLE_COUNT_1_BIT,
                    loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
                    storeOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                    stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
                    stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
                    initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
                    finalLayout: VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                };

                let depth_attachment_reference = VkAttachmentReference {
                    attachment: 1,
                    layout: VK_IMAGE_LAYOUT_DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                };

                let dependency = VkSubpassDependency {
                    srcSubpass: VK_SUBPASS_EXTERNAL as u32,
                    dstSubpass: 0,
                    srcStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
                        | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT,
                    dstStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT
                        | VK_PIPELINE_STAGE_EARLY_FRAGMENT_TESTS_BIT,
                    srcAccessMask: 0,
                    dstAccessMask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT
                        | VK_ACCESS_DEPTH_STENCIL_ATTACHMENT_WRITE_BIT,
                    dependencyFlags: 0,
                };

                (
                    vec![color_attachment_description, depth_attachment_description],
                    Some(depth_attachment_reference),
                    dependency,
                )
            } else {
                let dependency = VkSubpassDependency {
                    srcSubpass: VK_SUBPASS_EXTERNAL as u32,
                    dstSubpass: 0,
                    srcStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                    dstStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT,
                    srcAccessMask: 0,
                    dstAccessMask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT,
                    dependencyFlags: 0,
                };

                (vec![color_attachment_description], None, dependency)
            };

        let subpass_description = VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: core::ptr::null(),
            colorAttachmentCount: 1,
            pColorAttachments: &color_attachment_reference,
            pResolveAttachments: core::ptr::null(),
            pDepthStencilAttachment: depth_attachment_reference
                .as_ref()
                .map(|r| r as *const VkAttachmentReference)
                .unwrap_or(core::ptr::null()),
            preserveAttachmentCount: 0,
            pPreserveAttachments: core::ptr::null(),
        };

        let create_info = VkRenderPassCreateInfo {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            attachmentCount: attachments.len() as u32,
            pAttachments: attachments.as_ptr(),
            subpassCount: 1,
            pSubpasses: &subpass_description,
            dependencyCount: 1,
            pDependencies: &dependency,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreateRenderPass(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create render pass: {}", result));
            }
        }

        Ok(Self {
            handle,
            device: device.handle,
        })
    }

    /// Creates a custom render pass with explicit attachment and dependency configuration.
    ///
    /// This allows for more complex render pass configurations beyond the simple presets.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `attachments` - The attachment descriptions.
    /// * `color_refs` - The color attachment references.
    /// * `depth_ref` - Optional depth attachment reference.
    /// * `dependencies` - The subpass dependencies.
    pub fn new_custom(
        device: &VulkanDevice,
        attachments: &[VkAttachmentDescription],
        color_refs: &[VkAttachmentReference],
        depth_ref: Option<&VkAttachmentReference>,
        dependencies: &[VkSubpassDependency],
    ) -> Result<Self, String> {
        if attachments.is_empty() {
            return Err("Render pass must have at least one attachment".to_string());
        }

        let subpass_description = VkSubpassDescription {
            flags: 0,
            pipelineBindPoint: VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: core::ptr::null(),
            colorAttachmentCount: color_refs.len() as u32,
            pColorAttachments: if !color_refs.is_empty() {
                color_refs.as_ptr()
            } else {
                core::ptr::null()
            },
            pResolveAttachments: core::ptr::null(),
            pDepthStencilAttachment: depth_ref
                .map(|r| r as *const VkAttachmentReference)
                .unwrap_or(core::ptr::null()),
            preserveAttachmentCount: 0,
            pPreserveAttachments: core::ptr::null(),
        };

        let create_info = VkRenderPassCreateInfo {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            attachmentCount: attachments.len() as u32,
            pAttachments: attachments.as_ptr(),
            subpassCount: 1,
            pSubpasses: &subpass_description,
            dependencyCount: dependencies.len() as u32,
            pDependencies: if !dependencies.is_empty() {
                dependencies.as_ptr()
            } else {
                core::ptr::null()
            },
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreateRenderPass(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create render pass: {}", result));
            }
        }

        Ok(Self {
            handle,
            device: device.handle,
        })
    }
}

impl Drop for VulkanRenderPass {
    fn drop(&mut self) {
        unsafe {
            vkDestroyRenderPass(self.device, self.handle, core::ptr::null());
        }
    }
}
