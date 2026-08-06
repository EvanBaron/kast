use vk_bindings::*;

use crate::backend::vulkan::device::VulkanDevice;

/// A Vulkan pipeline wrapper handling both Graphics and Compute pipelines.
///
/// Wraps the Vulkan pipeline handle and its layout, ensuring proper cleanup on Drop.
pub struct VulkanPipeline {
    pub(crate) handle: VkPipeline,
    pub(crate) layout: VkPipelineLayout,
    pub(crate) bind_point: VkPipelineBindPoint,
    device: VkDevice,
}

impl VulkanPipeline {
    /// Creates a new graphics pipeline with the specified configuration.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `config` - The graphics pipeline configuration.
    /// * `layout` - The pipeline layout handle (ownership transferred to this struct).
    ///
    /// # Returns
    /// A new VulkanPipeline or an error if creation fails.
    pub fn new_graphics(
        device: &VulkanDevice,
        config: &GraphicsPipelineConfig,
        layout: VkPipelineLayout,
    ) -> Result<Self, String> {
        // Vertex input state
        let vertex_input_state_create_info = VkPipelineVertexInputStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            vertexBindingDescriptionCount: config.vertex_bindings.len() as u32,
            pVertexBindingDescriptions: config.vertex_bindings.as_ptr(),
            vertexAttributeDescriptionCount: config.vertex_attributes.len() as u32,
            pVertexAttributeDescriptions: config.vertex_attributes.as_ptr(),
        };

        // Input assembly state
        let input_assembly_state_create_info = VkPipelineInputAssemblyStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            topology: config.topology,
            primitiveRestartEnable: VK_FALSE,
        };

        // Viewport state (dynamic if using dynamic state)
        let viewport = VkViewport {
            x: 0.0,
            y: 0.0,
            width: config.viewport_extent.width as f32,
            height: config.viewport_extent.height as f32,
            minDepth: 0.0,
            maxDepth: 1.0,
        };

        let scissor = VkRect2D {
            offset: VkOffset2D { x: 0, y: 0 },
            extent: config.viewport_extent,
        };

        let viewport_state_create_info = VkPipelineViewportStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            viewportCount: 1,
            pViewports: &viewport,
            scissorCount: 1,
            pScissors: &scissor,
        };

        // Rasterization state
        let rasterization_state_create_info = VkPipelineRasterizationStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            depthClampEnable: VK_FALSE,
            rasterizerDiscardEnable: VK_FALSE,
            polygonMode: config.polygon_mode,
            cullMode: config.cull_mode,
            frontFace: config.front_face,
            depthBiasEnable: VK_FALSE,
            depthBiasConstantFactor: 0.0,
            depthBiasClamp: 0.0,
            depthBiasSlopeFactor: 0.0,
            lineWidth: 1.0,
        };

        // Multisample state
        let multisample_state_create_info = VkPipelineMultisampleStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            rasterizationSamples: VK_SAMPLE_COUNT_1_BIT,
            sampleShadingEnable: VK_FALSE,
            minSampleShading: 1.0,
            pSampleMask: core::ptr::null(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        };

        // Depth stencil state
        let depth_stencil_state_create_info = if config.depth_test_enable {
            VkPipelineDepthStencilStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                depthTestEnable: VK_TRUE,
                depthWriteEnable: if config.depth_write_enable {
                    VK_TRUE
                } else {
                    VK_FALSE
                },
                depthCompareOp: config.depth_compare_op,
                depthBoundsTestEnable: VK_FALSE,
                stencilTestEnable: VK_FALSE,
                front: VkStencilOpState::default(),
                back: VkStencilOpState::default(),
                minDepthBounds: 0.0,
                maxDepthBounds: 1.0,
            }
        } else {
            VkPipelineDepthStencilStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_DEPTH_STENCIL_STATE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                depthTestEnable: VK_FALSE,
                depthWriteEnable: VK_FALSE,
                depthCompareOp: VK_COMPARE_OP_NEVER,
                depthBoundsTestEnable: VK_FALSE,
                stencilTestEnable: VK_FALSE,
                front: VkStencilOpState::default(),
                back: VkStencilOpState::default(),
                minDepthBounds: 0.0,
                maxDepthBounds: 1.0,
            }
        };

        // Color blend state
        let color_blend_state_create_info = VkPipelineColorBlendStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            logicOpEnable: VK_FALSE,
            logicOp: VK_LOGIC_OP_COPY,
            attachmentCount: config.color_blend_attachments.len() as u32,
            pAttachments: config.color_blend_attachments.as_ptr(),
            blendConstants: [0.0, 0.0, 0.0, 0.0],
        };

        // Dynamic state
        let dynamic_state_create_info = if !config.dynamic_states.is_empty() {
            Some(VkPipelineDynamicStateCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                dynamicStateCount: config.dynamic_states.len() as u32,
                pDynamicStates: config.dynamic_states.as_ptr(),
            })
        } else {
            None
        };

        // Pipeline create info
        let create_info = VkGraphicsPipelineCreateInfo {
            sType: VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            stageCount: config.shader_stages.len() as u32,
            pStages: config.shader_stages.as_ptr(),
            pVertexInputState: &vertex_input_state_create_info,
            pInputAssemblyState: &input_assembly_state_create_info,
            pTessellationState: core::ptr::null(),
            pViewportState: &viewport_state_create_info,
            pRasterizationState: &rasterization_state_create_info,
            pMultisampleState: &multisample_state_create_info,
            pDepthStencilState: &depth_stencil_state_create_info,
            pColorBlendState: &color_blend_state_create_info,
            pDynamicState: dynamic_state_create_info
                .as_ref()
                .map(|s| s as *const VkPipelineDynamicStateCreateInfo)
                .unwrap_or(core::ptr::null()),
            layout,
            renderPass: config.render_pass,
            subpass: config.subpass,
            basePipelineHandle: core::ptr::null_mut(),
            basePipelineIndex: -1,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateGraphicsPipelines(
                device.handle,
                core::ptr::null_mut(),
                1,
                &create_info,
                core::ptr::null(),
                &mut handle,
            );
            if result != VK_SUCCESS {
                return Err(format!("Failed to create graphics pipeline: {}", result));
            }
        }

        Ok(Self {
            handle,
            layout,
            bind_point: VK_PIPELINE_BIND_POINT_GRAPHICS,
            device: device.handle,
        })
    }

    /// Creates a new compute pipeline with the specified shader and layout.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `shader_stage` - The compute shader stage create info.
    /// * `layout` - The pipeline layout handle (ownership transferred to this struct).
    ///
    /// # Returns
    /// A new VulkanPipeline or an error if creation fails.
    pub fn new_compute(
        device: &VulkanDevice,
        shader_stage: &VkPipelineShaderStageCreateInfo,
        layout: VkPipelineLayout,
    ) -> Result<Self, String> {
        let create_info = VkComputePipelineCreateInfo {
            sType: VK_STRUCTURE_TYPE_COMPUTE_PIPELINE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            stage: *shader_stage,
            layout,
            basePipelineHandle: core::ptr::null_mut(),
            basePipelineIndex: -1,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateComputePipelines(
                device.handle,
                core::ptr::null_mut(),
                1,
                &create_info,
                core::ptr::null(),
                &mut handle,
            );
            if result != VK_SUCCESS {
                return Err(format!("Failed to create compute pipeline: {}", result));
            }
        }

        Ok(Self {
            handle,
            layout,
            bind_point: VK_PIPELINE_BIND_POINT_COMPUTE,
            device: device.handle,
        })
    }
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        unsafe {
            vkDestroyPipeline(self.device, self.handle, core::ptr::null());
            vkDestroyPipelineLayout(self.device, self.layout, core::ptr::null());
        }
    }
}

/// A Vulkan pipeline layout defining shader resource bindings and push constants.
///
/// Pipeline layouts describe the interface between pipeline stages and shader resources,
/// including descriptor set layouts and push constant ranges.
pub struct VulkanPipelineLayout {
    pub(crate) handle: VkPipelineLayout,
    device: VkDevice,
}

impl VulkanPipelineLayout {
    /// Creates a new pipeline layout with descriptor sets and push constants.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `descriptor_set_layouts` - Array of descriptor set layouts to bind.
    /// * `push_constant_ranges` - Array of push constant ranges (can be empty).
    ///
    /// # Returns
    /// A new VulkanPipelineLayout or an error if creation fails.
    pub fn new(
        device: &VulkanDevice,
        descriptor_set_layouts: &[VkDescriptorSetLayout],
        push_constant_ranges: &[VkPushConstantRange],
    ) -> Result<Self, String> {
        let create_info = VkPipelineLayoutCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            setLayoutCount: descriptor_set_layouts.len() as u32,
            pSetLayouts: descriptor_set_layouts.as_ptr(),
            pushConstantRangeCount: push_constant_ranges.len() as u32,
            pPushConstantRanges: push_constant_ranges.as_ptr(),
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreatePipelineLayout(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create pipeline layout: {}", result));
            }
        }

        Ok(Self {
            handle,
            device: device.handle,
        })
    }
}

impl Drop for VulkanPipelineLayout {
    fn drop(&mut self) {
        unsafe {
            vkDestroyPipelineLayout(self.device, self.handle, core::ptr::null());
        }
    }
}

/// Configuration for creating a graphics pipeline.
///
/// This structure holds all the state needed to create a graphics pipeline.
/// Most fields have sensible defaults for common use cases.
pub struct GraphicsPipelineConfig<'a> {
    pub(crate) shader_stages: &'a [VkPipelineShaderStageCreateInfo],
    pub(crate) vertex_bindings: &'a [VkVertexInputBindingDescription],
    pub(crate) vertex_attributes: &'a [VkVertexInputAttributeDescription],
    pub(crate) topology: VkPrimitiveTopology,
    pub(crate) viewport_extent: VkExtent2D,
    pub(crate) polygon_mode: VkPolygonMode,
    pub(crate) cull_mode: VkCullModeFlags,
    pub(crate) front_face: VkFrontFace,
    pub depth_test_enable: bool,
    pub depth_write_enable: bool,
    pub(crate) depth_compare_op: VkCompareOp,
    pub(crate) color_blend_attachments: &'a [VkPipelineColorBlendAttachmentState],
    pub(crate) dynamic_states: &'a [VkDynamicState],
    pub(crate) pipeline_layout: VkPipelineLayout,
    pub(crate) render_pass: VkRenderPass,
    pub subpass: u32,
}

impl<'a> GraphicsPipelineConfig<'a> {
    /// Creates a default color blend attachment state with no blending.
    ///
    /// # Returns
    /// A color blend attachment configured for opaque rendering (no blending).
    pub fn default_color_blend_attachment() -> VkPipelineColorBlendAttachmentState {
        VkPipelineColorBlendAttachmentState {
            blendEnable: VK_FALSE,
            srcColorBlendFactor: VK_BLEND_FACTOR_ONE,
            dstColorBlendFactor: VK_BLEND_FACTOR_ZERO,
            colorBlendOp: VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: VK_BLEND_FACTOR_ONE,
            dstAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
            alphaBlendOp: VK_BLEND_OP_ADD,
            colorWriteMask: VK_COLOR_COMPONENT_R_BIT
                | VK_COLOR_COMPONENT_G_BIT
                | VK_COLOR_COMPONENT_B_BIT
                | VK_COLOR_COMPONENT_A_BIT,
        }
    }

    /// Creates a color blend attachment state with alpha blending.
    ///
    /// # Returns
    /// A color blend attachment configured for standard alpha blending.
    pub fn alpha_blend_attachment() -> VkPipelineColorBlendAttachmentState {
        VkPipelineColorBlendAttachmentState {
            blendEnable: VK_TRUE,
            srcColorBlendFactor: VK_BLEND_FACTOR_SRC_ALPHA,
            dstColorBlendFactor: VK_BLEND_FACTOR_ONE_MINUS_SRC_ALPHA,
            colorBlendOp: VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: VK_BLEND_FACTOR_ONE,
            dstAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
            alphaBlendOp: VK_BLEND_OP_ADD,
            colorWriteMask: VK_COLOR_COMPONENT_R_BIT
                | VK_COLOR_COMPONENT_G_BIT
                | VK_COLOR_COMPONENT_B_BIT
                | VK_COLOR_COMPONENT_A_BIT,
        }
    }
}

/// Creates a vertex binding description.
///
/// # Arguments
/// * `binding` - The binding number.
/// * `stride` - The byte stride between consecutive elements.
/// * `input_rate` - Whether to advance per vertex or per instance.
///
/// # Returns
/// A VkVertexInputBindingDescription.
pub fn binding_description(
    binding: u32,
    stride: u32,
    input_rate: VkVertexInputRate,
) -> VkVertexInputBindingDescription {
    VkVertexInputBindingDescription {
        binding,
        stride,
        inputRate: input_rate,
    }
}

/// Creates a vertex attribute description.
///
/// # Arguments
/// * `location` - The shader input location.
/// * `binding` - The binding number this attribute comes from.
/// * `format` - The format of the attribute data.
/// * `offset` - The byte offset of this attribute in the vertex structure.
///
/// # Returns
/// A VkVertexInputAttributeDescription.
pub fn attribute_description(
    location: u32,
    binding: u32,
    format: VkFormat,
    offset: u32,
) -> VkVertexInputAttributeDescription {
    VkVertexInputAttributeDescription {
        location,
        binding,
        format,
        offset,
    }
}
