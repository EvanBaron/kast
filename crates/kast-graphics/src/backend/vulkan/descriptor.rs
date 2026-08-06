use vk_bindings::*;

use crate::backend::vulkan::device::VulkanDevice;

pub const DEFAULT_MAX_TEXTURES: u32 = 1024;
pub const DEFAULT_MAX_SAMPLERS: u32 = 16;
pub const DEFAULT_MAX_UNIFORM_BUFFERS: u32 = 256;
pub const DEFAULT_MAX_STORAGE_BUFFERS: u32 = 256;

/// Descriptor indexing device limits for bindless descriptors.
#[derive(Debug, Clone, Copy)]
pub struct DescriptorIndexingLimits {
    pub max_sampled_images: u32,
    pub max_samplers: u32,
    pub max_uniform_buffers: u32,
    pub max_storage_buffers: u32,
}

impl DescriptorIndexingLimits {
    /// Queries descriptor indexing limits from the physical device.
    pub fn query(physical_device: VkPhysicalDevice) -> Self {
        let mut indexing_properties = VkPhysicalDeviceDescriptorIndexingProperties {
            sType: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_PROPERTIES,
            pNext: core::ptr::null_mut(),
            ..Default::default()
        };

        let mut properties2 = VkPhysicalDeviceProperties2 {
            sType: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_PROPERTIES_2,
            pNext: &mut indexing_properties as *mut _ as *mut core::ffi::c_void,
            properties: VkPhysicalDeviceProperties::default(),
        };

        unsafe {
            vkGetPhysicalDeviceProperties2(physical_device, &mut properties2);
        }

        Self {
            max_sampled_images: indexing_properties.maxDescriptorSetUpdateAfterBindSampledImages,
            max_samplers: indexing_properties.maxDescriptorSetUpdateAfterBindSamplers,
            max_uniform_buffers: indexing_properties.maxDescriptorSetUpdateAfterBindUniformBuffers,
            max_storage_buffers: indexing_properties.maxDescriptorSetUpdateAfterBindStorageBuffers,
        }
    }
}

/// Configuration for bindless descriptor system
#[derive(Debug, Clone)]
pub struct BindlessDescriptorConfig {
    pub max_textures: u32,
    pub max_samplers: u32,
    pub max_uniform_buffers: u32,
    pub max_storage_buffers: u32,
}

impl Default for BindlessDescriptorConfig {
    fn default() -> Self {
        Self {
            max_textures: DEFAULT_MAX_TEXTURES,
            max_samplers: DEFAULT_MAX_SAMPLERS,
            max_uniform_buffers: DEFAULT_MAX_UNIFORM_BUFFERS,
            max_storage_buffers: DEFAULT_MAX_STORAGE_BUFFERS,
        }
    }
}

/// Slot allocator for managing descriptor array indices
struct SlotAllocator {
    free_slots: Vec<u32>,
    next_slot: u32,
    max_slots: u32,
}

impl SlotAllocator {
    fn new(max_slots: u32) -> Self {
        Self {
            free_slots: Vec::new(),
            next_slot: 0,
            max_slots,
        }
    }

    /// Allocates a slot, returns None if all slots are in use
    fn allocate(&mut self) -> Option<u32> {
        if let Some(slot) = self.free_slots.pop() {
            Some(slot)
        } else if self.next_slot < self.max_slots {
            let slot = self.next_slot;
            self.next_slot += 1;
            Some(slot)
        } else {
            None
        }
    }

    /// Frees a previously allocated slot
    fn free(&mut self, slot: u32) {
        if slot < self.max_slots {
            self.free_slots.push(slot);
        }
    }

    /// Returns the number of allocated slots
    fn allocated_count(&self) -> u32 {
        self.next_slot - self.free_slots.len() as u32
    }
}

/// Bindless descriptor system for efficient dynamic descriptor management.
/// Uses descriptor indexing to expose large arrays that shaders can index freely.
///
/// # Binding layout (Set 0)
///
/// | Binding | Type            | Max count              | Flags                         |
/// |---------|-----------------|------------------------|-------------------------------|
/// | 0       | Sampler         | max_samplers           | PARTIALLY_BOUND, UAB          |
/// | 1       | Uniform buffer  | max_uniform_buffers    | PARTIALLY_BOUND, UAB          |
/// | 2       | Storage buffer  | max_storage_buffers    | PARTIALLY_BOUND, UAB          |
/// | 3       | Sampled image   | max_textures           | PARTIALLY_BOUND, UAB, VARIABLE|
///
/// UAB = UPDATE_AFTER_BIND
pub struct BindlessDescriptorSystem {
    device: VkDevice,
    descriptor_set_layout: VkDescriptorSetLayout,
    descriptor_pool: VkDescriptorPool,
    descriptor_set: VkDescriptorSet,
    texture_slots: SlotAllocator,
    sampler_slots: SlotAllocator,
    uniform_buffer_slots: SlotAllocator,
    storage_buffer_slots: SlotAllocator,
    config: BindlessDescriptorConfig,
}

impl BindlessDescriptorSystem {
    /// Creates a new bindless descriptor system.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device (must have descriptor indexing features enabled)
    /// * `config` - Configuration for descriptor counts (will be clamped to device limits)
    ///
    /// # Returns
    /// A new BindlessDescriptorSystem or an error
    pub fn new(device: &VulkanDevice, config: BindlessDescriptorConfig) -> Result<Self, String> {
        let limits = DescriptorIndexingLimits::query(device.physical_device);
        let config = BindlessDescriptorConfig {
            max_textures: config.max_textures.min(limits.max_sampled_images),
            max_samplers: config.max_samplers.min(limits.max_samplers),
            max_uniform_buffers: config.max_uniform_buffers.min(limits.max_uniform_buffers),
            max_storage_buffers: config.max_storage_buffers.min(limits.max_storage_buffers),
        };

        let binding_flags = [
            // Binding 0: Samplers
            VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT | VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT,
            // Binding 1: Uniform buffers
            VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT | VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT,
            // Binding 2: Storage buffers
            VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT | VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT,
            // Binding 3: Sampled images
            VK_DESCRIPTOR_BINDING_PARTIALLY_BOUND_BIT
                | VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT
                | VK_DESCRIPTOR_BINDING_VARIABLE_DESCRIPTOR_COUNT_BIT,
        ];

        let bindings = [
            // Binding 0: Samplers
            VkDescriptorSetLayoutBinding {
                binding: 0,
                descriptorType: VK_DESCRIPTOR_TYPE_SAMPLER,
                descriptorCount: config.max_samplers,
                stageFlags: VK_SHADER_STAGE_FRAGMENT_BIT | VK_SHADER_STAGE_COMPUTE_BIT,
                pImmutableSamplers: core::ptr::null(),
            },
            // Binding 1: Uniform buffers
            VkDescriptorSetLayoutBinding {
                binding: 1,
                descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: config.max_uniform_buffers,
                stageFlags: VK_SHADER_STAGE_ALL,
                pImmutableSamplers: core::ptr::null(),
            },
            // Binding 2: Storage buffers
            VkDescriptorSetLayoutBinding {
                binding: 2,
                descriptorType: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                descriptorCount: config.max_storage_buffers,
                stageFlags: VK_SHADER_STAGE_ALL,
                pImmutableSamplers: core::ptr::null(),
            },
            // Binding 3: Sampled image
            VkDescriptorSetLayoutBinding {
                binding: 3,
                descriptorType: VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                descriptorCount: config.max_textures,
                stageFlags: VK_SHADER_STAGE_FRAGMENT_BIT | VK_SHADER_STAGE_COMPUTE_BIT,
                pImmutableSamplers: core::ptr::null(),
            },
        ];

        let binding_flags_create_info = VkDescriptorSetLayoutBindingFlagsCreateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_BINDING_FLAGS_CREATE_INFO,
            pNext: core::ptr::null(),
            bindingCount: binding_flags.len() as u32,
            pBindingFlags: binding_flags.as_ptr(),
        };

        let layout_create_info = VkDescriptorSetLayoutCreateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            pNext: &binding_flags_create_info as *const _ as *const core::ffi::c_void,
            flags: VK_DESCRIPTOR_SET_LAYOUT_CREATE_UPDATE_AFTER_BIND_POOL_BIT,
            bindingCount: bindings.len() as u32,
            pBindings: bindings.as_ptr(),
        };

        let mut descriptor_set_layout = core::ptr::null_mut();
        unsafe {
            let result = vkCreateDescriptorSetLayout(
                device.handle,
                &layout_create_info,
                core::ptr::null(),
                &mut descriptor_set_layout,
            );
            if result != VK_SUCCESS {
                return Err(format!(
                    "Failed to create bindless descriptor set layout: {}",
                    result
                ));
            }
        }

        // Create descriptor pool
        let pool_sizes = [
            VkDescriptorPoolSize {
                type_: VK_DESCRIPTOR_TYPE_SAMPLER,
                descriptorCount: config.max_samplers,
            },
            VkDescriptorPoolSize {
                type_: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: config.max_uniform_buffers,
            },
            VkDescriptorPoolSize {
                type_: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                descriptorCount: config.max_storage_buffers,
            },
            VkDescriptorPoolSize {
                type_: VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
                descriptorCount: config.max_textures,
            },
        ];

        let pool_create_info = VkDescriptorPoolCreateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: VK_DESCRIPTOR_POOL_CREATE_UPDATE_AFTER_BIND_BIT,
            maxSets: 1,
            poolSizeCount: pool_sizes.len() as u32,
            pPoolSizes: pool_sizes.as_ptr(),
        };

        let mut descriptor_pool = core::ptr::null_mut();
        unsafe {
            let result = vkCreateDescriptorPool(
                device.handle,
                &pool_create_info,
                core::ptr::null(),
                &mut descriptor_pool,
            );
            if result != VK_SUCCESS {
                vkDestroyDescriptorSetLayout(
                    device.handle,
                    descriptor_set_layout,
                    core::ptr::null(),
                );
                return Err(format!(
                    "Failed to create bindless descriptor pool: {}",
                    result
                ));
            }
        }

        // Allocate descriptor set with variable descriptor count
        let variable_descriptor_count = config.max_textures;
        let variable_descriptor_count_info = VkDescriptorSetVariableDescriptorCountAllocateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_VARIABLE_DESCRIPTOR_COUNT_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            descriptorSetCount: 1,
            pDescriptorCounts: &variable_descriptor_count,
        };

        let allocate_info = VkDescriptorSetAllocateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: &variable_descriptor_count_info as *const _ as *const core::ffi::c_void,
            descriptorPool: descriptor_pool,
            descriptorSetCount: 1,
            pSetLayouts: &descriptor_set_layout,
        };

        let mut descriptor_set = core::ptr::null_mut();
        unsafe {
            let result =
                vkAllocateDescriptorSets(device.handle, &allocate_info, &mut descriptor_set);
            if result != VK_SUCCESS {
                vkDestroyDescriptorPool(device.handle, descriptor_pool, core::ptr::null());
                vkDestroyDescriptorSetLayout(
                    device.handle,
                    descriptor_set_layout,
                    core::ptr::null(),
                );
                return Err(format!(
                    "Failed to allocate bindless descriptor set: {}",
                    result
                ));
            }
        }

        Ok(Self {
            device: device.handle,
            descriptor_set_layout,
            descriptor_pool,
            descriptor_set,
            texture_slots: SlotAllocator::new(config.max_textures),
            sampler_slots: SlotAllocator::new(config.max_samplers),
            uniform_buffer_slots: SlotAllocator::new(config.max_uniform_buffers),
            storage_buffer_slots: SlotAllocator::new(config.max_storage_buffers),
            config,
        })
    }

    /// Allocates a texture slot
    ///
    /// # Returns
    /// The slot index or None if all slots are in use
    pub fn allocate_texture_slot(&mut self) -> Option<u32> {
        self.texture_slots.allocate()
    }

    /// Frees a texture slot
    pub fn free_texture_slot(&mut self, slot: u32) {
        self.texture_slots.free(slot);
    }

    /// Allocates a sampler slot
    ///
    /// # Returns
    /// The slot index or None if all slots are in use
    pub fn allocate_sampler_slot(&mut self) -> Option<u32> {
        self.sampler_slots.allocate()
    }

    /// Frees a sampler slot
    pub fn free_sampler_slot(&mut self, slot: u32) {
        self.sampler_slots.free(slot);
    }

    /// Allocates a uniform buffer slot
    ///
    /// # Returns
    /// The slot index or None if all slots are in use
    pub fn allocate_uniform_buffer_slot(&mut self) -> Option<u32> {
        self.uniform_buffer_slots.allocate()
    }

    /// Frees a uniform buffer slot
    pub fn free_uniform_buffer_slot(&mut self, slot: u32) {
        self.uniform_buffer_slots.free(slot);
    }

    /// Allocates a storage buffer slot
    ///
    /// # Returns
    /// The slot index or None if all slots are in use
    pub fn allocate_storage_buffer_slot(&mut self) -> Option<u32> {
        self.storage_buffer_slots.allocate()
    }

    /// Frees a storage buffer slot
    pub fn free_storage_buffer_slot(&mut self, slot: u32) {
        self.storage_buffer_slots.free(slot);
    }

    /// Updates a texture slot with an image view
    ///
    /// # Arguments
    /// * `slot` - The slot index
    /// * `image_view` - The image view to bind
    /// * `layout` - The image layout
    pub fn update_texture(&self, slot: u32, image_view: VkImageView, layout: VkImageLayout) {
        let image_info = VkDescriptorImageInfo {
            sampler: core::ptr::null_mut(),
            imageView: image_view,
            imageLayout: layout,
        };

        let write = VkWriteDescriptorSet {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: self.descriptor_set,
            dstBinding: 3,
            dstArrayElement: slot,
            descriptorCount: 1,
            descriptorType: VK_DESCRIPTOR_TYPE_SAMPLED_IMAGE,
            pImageInfo: &image_info,
            pBufferInfo: core::ptr::null(),
            pTexelBufferView: core::ptr::null(),
        };

        unsafe {
            vkUpdateDescriptorSets(self.device, 1, &write, 0, core::ptr::null());
        }
    }

    /// Updates a sampler slot
    ///
    /// # Arguments
    /// * `slot` - The slot index
    /// * `sampler` - The sampler to bind
    pub fn update_sampler(&self, slot: u32, sampler: VkSampler) {
        let image_info = VkDescriptorImageInfo {
            sampler,
            imageView: core::ptr::null_mut(),
            imageLayout: VK_IMAGE_LAYOUT_UNDEFINED,
        };

        let write = VkWriteDescriptorSet {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: self.descriptor_set,
            dstBinding: 0,
            dstArrayElement: slot,
            descriptorCount: 1,
            descriptorType: VK_DESCRIPTOR_TYPE_SAMPLER,
            pImageInfo: &image_info,
            pBufferInfo: core::ptr::null(),
            pTexelBufferView: core::ptr::null(),
        };

        unsafe {
            vkUpdateDescriptorSets(self.device, 1, &write, 0, core::ptr::null());
        }
    }

    /// Updates a uniform buffer slot
    ///
    /// # Arguments
    /// * `slot` - The slot index
    /// * `buffer` - The buffer to bind
    /// * `offset` - Offset in the buffer
    /// * `range` - Range of the buffer
    pub fn update_uniform_buffer(&self, slot: u32, buffer: VkBuffer, offset: u64, range: u64) {
        let buffer_info = VkDescriptorBufferInfo {
            buffer,
            offset,
            range,
        };

        let write = VkWriteDescriptorSet {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: self.descriptor_set,
            dstBinding: 1,
            dstArrayElement: slot,
            descriptorCount: 1,
            descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
            pImageInfo: core::ptr::null(),
            pBufferInfo: &buffer_info,
            pTexelBufferView: core::ptr::null(),
        };

        unsafe {
            vkUpdateDescriptorSets(self.device, 1, &write, 0, core::ptr::null());
        }
    }

    /// Updates a storage buffer slot
    ///
    /// # Arguments
    /// * `slot` - The slot index
    /// * `buffer` - The buffer to bind
    /// * `offset` - Offset in the buffer
    /// * `range` - Range of the buffer
    pub fn update_storage_buffer(&self, slot: u32, buffer: VkBuffer, offset: u64, range: u64) {
        let buffer_info = VkDescriptorBufferInfo {
            buffer,
            offset,
            range,
        };

        let write = VkWriteDescriptorSet {
            sType: VK_STRUCTURE_TYPE_WRITE_DESCRIPTOR_SET,
            pNext: core::ptr::null(),
            dstSet: self.descriptor_set,
            dstBinding: 2,
            dstArrayElement: slot,
            descriptorCount: 1,
            descriptorType: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
            pImageInfo: core::ptr::null(),
            pBufferInfo: &buffer_info,
            pTexelBufferView: core::ptr::null(),
        };

        unsafe {
            vkUpdateDescriptorSets(self.device, 1, &write, 0, core::ptr::null());
        }
    }

    /// Returns the descriptor set layout
    pub fn layout(&self) -> VkDescriptorSetLayout {
        self.descriptor_set_layout
    }

    /// Returns the descriptor set
    pub fn descriptor_set(&self) -> VkDescriptorSet {
        self.descriptor_set
    }

    /// Returns statistics about slot usage
    pub fn stats(&self) -> BindlessDescriptorStats {
        BindlessDescriptorStats {
            textures_allocated: self.texture_slots.allocated_count(),
            textures_max: self.config.max_textures,
            samplers_allocated: self.sampler_slots.allocated_count(),
            samplers_max: self.config.max_samplers,
            uniform_buffers_allocated: self.uniform_buffer_slots.allocated_count(),
            uniform_buffers_max: self.config.max_uniform_buffers,
            storage_buffers_allocated: self.storage_buffer_slots.allocated_count(),
            storage_buffers_max: self.config.max_storage_buffers,
        }
    }
}

impl Drop for BindlessDescriptorSystem {
    fn drop(&mut self) {
        unsafe {
            vkDestroyDescriptorPool(self.device, self.descriptor_pool, core::ptr::null());
            vkDestroyDescriptorSetLayout(
                self.device,
                self.descriptor_set_layout,
                core::ptr::null(),
            );
        }
    }
}

/// Statistics about bindless descriptor usage
#[derive(Debug, Clone, Copy)]
pub struct BindlessDescriptorStats {
    pub textures_allocated: u32,
    pub textures_max: u32,
    pub samplers_allocated: u32,
    pub samplers_max: u32,
    pub uniform_buffers_allocated: u32,
    pub uniform_buffers_max: u32,
    pub storage_buffers_allocated: u32,
    pub storage_buffers_max: u32,
}
