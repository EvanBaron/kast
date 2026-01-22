use std::io::Read;
use vk_bindings::*;

use crate::graphics::mesh::Vertex;

pub struct Pipeline {
    pub handle: VkPipeline,
    pub layout: VkPipelineLayout,
    device: VkDevice,
}

impl Pipeline {
    /// Creates a new pipeline instance.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `render_pass` - The Vulkan render pass.
    pub fn new(device: VkDevice, render_pass: VkRenderPass) -> Self {
        let vert_shader_code = read_shader_file("shaders/01_attribute_position.vert.spv");
        let frag_shader_code = read_shader_file("shaders/00_hardcoded_red.frag.spv");

        let vert_shader_module = create_shader_module(device, &vert_shader_code);
        let frag_shader_module = create_shader_module(device, &frag_shader_code);

        // Pipeline Layout
        let pipeline_layout_create_info = VkPipelineLayoutCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            setLayoutCount: 0,
            pSetLayouts: core::ptr::null(),
            pushConstantRangeCount: 0,
            pPushConstantRanges: core::ptr::null(),
        };

        println!("Creating pipeline layout.");
        let mut pipeline_layout = core::ptr::null_mut();
        let result = unsafe {
            vkCreatePipelineLayout(
                device,
                &pipeline_layout_create_info,
                core::ptr::null(),
                &mut pipeline_layout,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create pipeline layout. Error: {}", result);
        }

        let shader_stages_info = [
            VkPipelineShaderStageCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                stage: VK_SHADER_STAGE_VERTEX_BIT,
                module: vert_shader_module,
                pName: b"main\0".as_ptr() as *const i8,
                pSpecializationInfo: core::ptr::null(),
            },
            VkPipelineShaderStageCreateInfo {
                sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                stage: VK_SHADER_STAGE_FRAGMENT_BIT,
                module: frag_shader_module,
                pName: b"main\0".as_ptr() as *const i8,
                pSpecializationInfo: core::ptr::null(),
            },
        ];

        let vertex_binding_description = Vertex::get_binding_description();
        let vertex_bindings = [vertex_binding_description];

        let vertex_attribute_descriptions = Vertex::get_attribute_descriptions();
        let vertex_attributes = vertex_attribute_descriptions;

        // Vertex Input
        let vertex_input_state_create_info = VkPipelineVertexInputStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VERTEX_INPUT_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            vertexBindingDescriptionCount: vertex_bindings.len() as u32,
            pVertexBindingDescriptions: vertex_bindings.as_ptr(),
            vertexAttributeDescriptionCount: vertex_attributes.len() as u32,
            pVertexAttributeDescriptions: vertex_attributes.as_ptr(),
        };

        // Input Assembly
        let input_assembly_create_info = VkPipelineInputAssemblyStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_INPUT_ASSEMBLY_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            topology: VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            primitiveRestartEnable: VK_FALSE,
        };

        // Dynamic States
        let dynamic_states = [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR];

        let dynamic_state_create_info = VkPipelineDynamicStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_DYNAMIC_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            dynamicStateCount: dynamic_states.len() as u32,
            pDynamicStates: dynamic_states.as_ptr(),
        };

        let dynamic_viewport_state_create_info = VkPipelineViewportStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_VIEWPORT_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            viewportCount: 1,
            pViewports: core::ptr::null(),
            scissorCount: 1,
            pScissors: core::ptr::null(),
        };

        // Rasterizer
        let rasterizer_create_info = VkPipelineRasterizationStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_RASTERIZATION_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            depthClampEnable: VK_FALSE,
            rasterizerDiscardEnable: VK_FALSE,
            polygonMode: VK_POLYGON_MODE_FILL,
            cullMode: VK_CULL_MODE_BACK_BIT as VkCullModeFlags,
            frontFace: VK_FRONT_FACE_CLOCKWISE,
            depthBiasEnable: VK_FALSE,
            depthBiasConstantFactor: 0.0,
            depthBiasClamp: 0.0,
            depthBiasSlopeFactor: 0.0,
            lineWidth: 1.0,
        };

        // Multisampling
        let multisampling_create_info = VkPipelineMultisampleStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_MULTISAMPLE_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            sampleShadingEnable: VK_FALSE,
            rasterizationSamples: VK_SAMPLE_COUNT_1_BIT,
            minSampleShading: 1.0,
            pSampleMask: core::ptr::null(),
            alphaToCoverageEnable: VK_FALSE,
            alphaToOneEnable: VK_FALSE,
        };

        // Color Blending
        let color_blend_attachment_create_info = VkPipelineColorBlendAttachmentState {
            colorWriteMask: (VK_COLOR_COMPONENT_R_BIT
                | VK_COLOR_COMPONENT_G_BIT
                | VK_COLOR_COMPONENT_B_BIT
                | VK_COLOR_COMPONENT_A_BIT) as VkColorComponentFlags,
            blendEnable: VK_FALSE,
            srcColorBlendFactor: VK_BLEND_FACTOR_ZERO,
            dstColorBlendFactor: VK_BLEND_FACTOR_ZERO,
            colorBlendOp: VK_BLEND_OP_ADD,
            srcAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
            dstAlphaBlendFactor: VK_BLEND_FACTOR_ZERO,
            alphaBlendOp: VK_BLEND_OP_ADD,
        };

        let color_blending_create_info = VkPipelineColorBlendStateCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_COLOR_BLEND_STATE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            logicOpEnable: VK_FALSE,
            logicOp: VK_LOGIC_OP_CLEAR,
            attachmentCount: 1,
            pAttachments: &color_blend_attachment_create_info,
            blendConstants: [0.0, 0.0, 0.0, 0.0],
        };

        // Graphics Pipeline Creation
        let pipelines_create_infos = [VkGraphicsPipelineCreateInfo {
            sType: VK_STRUCTURE_TYPE_GRAPHICS_PIPELINE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            stageCount: shader_stages_info.len() as u32,
            pStages: shader_stages_info.as_ptr(),
            pVertexInputState: &vertex_input_state_create_info,
            pInputAssemblyState: &input_assembly_create_info,
            pTessellationState: core::ptr::null(),
            pViewportState: &dynamic_viewport_state_create_info,
            pRasterizationState: &rasterizer_create_info,
            pMultisampleState: &multisampling_create_info,
            pDepthStencilState: core::ptr::null(),
            pColorBlendState: &color_blending_create_info,
            pDynamicState: &dynamic_state_create_info,
            layout: pipeline_layout,
            renderPass: render_pass,
            subpass: 0,
            basePipelineHandle: core::ptr::null_mut(),
            basePipelineIndex: -1,
        }];

        println!("Creating pipeline.");
        let mut graphics_pipelines = core::ptr::null_mut();
        let result = unsafe {
            vkCreateGraphicsPipelines(
                device,
                core::ptr::null_mut(),
                pipelines_create_infos.len() as u32,
                pipelines_create_infos.as_ptr(),
                core::ptr::null(),
                &mut graphics_pipelines,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create graphics pipeline!");
        }

        // Cleanup shader modules after pipeline creation
        unsafe {
            vkDestroyShaderModule(device, vert_shader_module, core::ptr::null());
            vkDestroyShaderModule(device, frag_shader_module, core::ptr::null());
        }

        Self {
            device,
            handle: graphics_pipelines,
            layout: pipeline_layout,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            vkDestroyPipeline(self.device, self.handle, core::ptr::null());
            vkDestroyPipelineLayout(self.device, self.layout, core::ptr::null());
        }
    }
}

/// Helper function to read shader bytecode from a file.
///
/// # Arguments
/// * `file_path` - The path to the shader file.
fn read_shader_file(file_path: &str) -> Vec<u8> {
    let mut file = std::fs::File::open(file_path).expect("Failed to open shader file");
    let mut bytecode = Vec::new();
    file.read_to_end(&mut bytecode)
        .expect("Failed to read shader file");

    bytecode
}

/// Helper function to create a shader module from bytecode.
///
/// # Arguments
/// * `device` - The Vulkan device.
/// * `code` - The shader bytecode.
fn create_shader_module(device: VkDevice, code: &[u8]) -> VkShaderModule {
    let create_info = VkShaderModuleCreateInfo {
        sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
        pNext: core::ptr::null(),
        flags: 0x0,
        codeSize: code.len(),
        pCode: code.as_ptr() as *const u32,
    };

    let mut module = core::ptr::null_mut();
    let result =
        unsafe { vkCreateShaderModule(device, &create_info, core::ptr::null(), &mut module) };

    if result != VK_SUCCESS {
        panic!("Failed to create pipeline. Error: {}", result);
    }

    module
}
