use vk_bindings::*;

use crate::backend::vulkan::device::VulkanDevice;

/// A Vulkan sampler for texture sampling operations.
///
/// Samplers define how textures are sampled in shaders, including filtering modes
/// (nearest, linear), address modes (repeat, clamp, etc.), and other sampling parameters.
/// Automatically destroyed when dropped.
pub struct VulkanSampler {
    pub(crate) handle: VkSampler,
    pub(crate) bindless_slot: Option<u32>,
    device: VkDevice,
}

impl VulkanSampler {
    /// Creates a new sampler with the specified filtering and address modes.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `mag_filter` - Magnification filter.
    /// * `min_filter` - Minification filter.
    /// * `address_mode_u` - Address mode for U coordinates.
    /// * `address_mode_v` - Address mode for V coordinates.
    /// * `address_mode_w` - Address mode for W coordinates.
    /// * `anisotropy_enable` - Whether to enable anisotropic filtering.
    /// * `max_anisotropy` - Maximum anisotropy level (typically 1.0 to 16.0).
    ///
    /// # Returns
    /// A new VulkanSampler or an error if creation fails.
    pub fn new(
        device: &VulkanDevice,
        mag_filter: VkFilter,
        min_filter: VkFilter,
        address_mode_u: VkSamplerAddressMode,
        address_mode_v: VkSamplerAddressMode,
        address_mode_w: VkSamplerAddressMode,
        anisotropy_enable: bool,
        max_anisotropy: f32,
    ) -> Result<Self, String> {
        let create_info = VkSamplerCreateInfo {
            sType: VK_STRUCTURE_TYPE_SAMPLER_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            magFilter: mag_filter,
            minFilter: min_filter,
            mipmapMode: VK_SAMPLER_MIPMAP_MODE_LINEAR,
            addressModeU: address_mode_u,
            addressModeV: address_mode_v,
            addressModeW: address_mode_w,
            mipLodBias: 0.0,
            anisotropyEnable: if anisotropy_enable { VK_TRUE } else { VK_FALSE },
            maxAnisotropy: max_anisotropy,
            compareEnable: VK_FALSE,
            compareOp: VK_COMPARE_OP_ALWAYS,
            minLod: 0.0,
            maxLod: VK_LOD_CLAMP_NONE as f32,
            borderColor: VK_BORDER_COLOR_INT_OPAQUE_BLACK,
            unnormalizedCoordinates: VK_FALSE,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreateSampler(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create sampler: {}", result));
            }
        }

        Ok(Self {
            handle,
            device: device.handle,
            bindless_slot: None,
        })
    }

    /// Creates a sampler with linear filtering and repeat address mode.
    ///
    /// This is a common configuration for repeating textures with smooth filtering.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    ///
    /// # Returns
    /// A new VulkanSampler configured for linear filtering with repeat mode.
    pub fn new_linear_repeat(device: &VulkanDevice) -> Result<Self, String> {
        Self::new(
            device,
            VK_FILTER_LINEAR,
            VK_FILTER_LINEAR,
            VK_SAMPLER_ADDRESS_MODE_REPEAT,
            VK_SAMPLER_ADDRESS_MODE_REPEAT,
            VK_SAMPLER_ADDRESS_MODE_REPEAT,
            false,
            1.0,
        )
    }

    /// Creates a sampler with nearest filtering and clamp address mode.
    ///
    /// This is useful for pixel-art textures or UI elements where you want sharp pixels.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    ///
    /// # Returns
    /// A new VulkanSampler configured for nearest filtering with clamp mode.
    pub fn new_nearest_clamp(device: &VulkanDevice) -> Result<Self, String> {
        Self::new(
            device,
            VK_FILTER_NEAREST,
            VK_FILTER_NEAREST,
            VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
            VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
            VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
            false,
            1.0,
        )
    }

    /// Creates a sampler with linear filtering, repeat mode, and anisotropic filtering.
    ///
    /// This provides the highest quality texture sampling, suitable for 3D scenes.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `max_anisotropy` - Maximum anisotropy level (typically 16.0 for high quality).
    ///
    /// # Returns
    /// A new VulkanSampler configured for high-quality anisotropic filtering.
    pub fn new_anisotropic(device: &VulkanDevice, max_anisotropy: f32) -> Result<Self, String> {
        Self::new(
            device,
            VK_FILTER_LINEAR,
            VK_FILTER_LINEAR,
            VK_SAMPLER_ADDRESS_MODE_REPEAT,
            VK_SAMPLER_ADDRESS_MODE_REPEAT,
            VK_SAMPLER_ADDRESS_MODE_REPEAT,
            true,
            max_anisotropy,
        )
    }
}

impl Drop for VulkanSampler {
    fn drop(&mut self) {
        unsafe {
            vkDestroySampler(self.device, self.handle, core::ptr::null());
        }
    }
}
