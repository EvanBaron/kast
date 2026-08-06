use vk_bindings::*;

use crate::backend::vulkan::device::VulkanDevice;

/// A Vulkan shader module containing SPIR-V bytecode.
///
/// Shader modules are created from SPIR-V bytecode and are used when creating
/// graphics or compute pipelines. They are typically destroyed after pipeline
/// creation is complete.
pub struct VulkanShader {
    pub(crate) handle: VkShaderModule,
    device: VkDevice,
}

impl VulkanShader {
    /// Creates a new shader module from SPIR-V bytecode.
    ///
    /// The bytecode must be valid SPIR-V and properly aligned to 4-byte boundaries.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `bytecode` - The SPIR-V bytecode as a byte slice.
    ///
    /// # Returns
    /// A new VulkanShader or an error if shader module creation fails.
    pub fn new(device: &VulkanDevice, bytecode: &[u8]) -> Result<Self, String> {
        if bytecode.len() % 4 != 0 {
            return Err("SPIR-V bytecode must be aligned to 4 bytes".to_string());
        }

        if bytecode.is_empty() {
            return Err("SPIR-V bytecode cannot be empty".to_string());
        }

        let create_info = VkShaderModuleCreateInfo {
            sType: VK_STRUCTURE_TYPE_SHADER_MODULE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            codeSize: bytecode.len(),
            pCode: bytecode.as_ptr() as *const u32,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreateShaderModule(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create shader module: {}", result));
            }
        }

        Ok(Self {
            handle,
            device: device.handle,
        })
    }

    /// Creates a new shader module from a SPIR-V file.
    ///
    /// This is a convenience function that reads the file and calls `new()`.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `file_path` - The path to the SPIR-V shader file.
    ///
    /// # Returns
    /// A new VulkanShader or an error if file reading or shader creation fails.
    pub fn from_file(device: &VulkanDevice, file_path: &str) -> Result<Self, String> {
        let bytecode = std::fs::read(file_path)
            .map_err(|e| format!("Failed to read shader file '{}': {}", file_path, e))?;

        Self::new(device, &bytecode)
    }

    /// Creates a shader stage create info for use in pipeline creation.
    ///
    /// # Arguments
    /// * `stage` - The shader stage.
    /// * `entry_point` - The name of the entry point function (typically "main").
    ///
    /// # Returns
    /// A VkPipelineShaderStageCreateInfo configured for this shader module.
    pub fn create_stage_info(
        &self,
        stage: VkShaderStageFlagBits,
        entry_point: &std::ffi::CStr,
    ) -> VkPipelineShaderStageCreateInfo {
        VkPipelineShaderStageCreateInfo {
            sType: VK_STRUCTURE_TYPE_PIPELINE_SHADER_STAGE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            stage,
            module: self.handle,
            pName: entry_point.as_ptr(),
            pSpecializationInfo: core::ptr::null(),
        }
    }
}

impl Drop for VulkanShader {
    fn drop(&mut self) {
        unsafe {
            vkDestroyShaderModule(self.device, self.handle, core::ptr::null());
        }
    }
}
