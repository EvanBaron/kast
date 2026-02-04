use vk_bindings::*;

pub struct DescriptorSetLayout {
    pub handle: VkDescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(device: VkDevice) -> Self {
        let descriptor_set_layout_bindings = [
            VkDescriptorSetLayoutBinding {
                binding: 0,
                descriptorType: VK_DESCRIPTOR_TYPE_UNIFORM_BUFFER,
                descriptorCount: 1,
                stageFlags: VK_SHADER_STAGE_VERTEX_BIT as VkShaderStageFlags,
                pImmutableSamplers: core::ptr::null(),
            },
            VkDescriptorSetLayoutBinding {
                binding: 1,
                descriptorType: VK_DESCRIPTOR_TYPE_STORAGE_BUFFER,
                descriptorCount: 1,
                stageFlags: VK_SHADER_STAGE_VERTEX_BIT as VkShaderStageFlags,
                pImmutableSamplers: core::ptr::null(),
            },
            VkDescriptorSetLayoutBinding {
                binding: 2,
                descriptorType: VK_DESCRIPTOR_TYPE_COMBINED_IMAGE_SAMPLER,
                descriptorCount: 16,
                stageFlags: VK_SHADER_STAGE_FRAGMENT_BIT as VkShaderStageFlags,
                pImmutableSamplers: core::ptr::null(),
            },
        ];

        let descriptor_set_layout_create_info = VkDescriptorSetLayoutCreateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_LAYOUT_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            bindingCount: descriptor_set_layout_bindings.len() as u32,
            pBindings: descriptor_set_layout_bindings.as_ptr(),
        };

        let mut handle = core::ptr::null_mut();
        let result = unsafe {
            vkCreateDescriptorSetLayout(
                device,
                &descriptor_set_layout_create_info,
                core::ptr::null_mut(),
                &mut handle,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create descriptor set layout. Error: {}.", result);
        }

        Self { handle }
    }

    pub fn destroy(&mut self, device: VkDevice) {
        unsafe {
            vkDestroyDescriptorSetLayout(device, self.handle, core::ptr::null());
        }
    }
}

pub struct DescriptorPool {
    pub handle: VkDescriptorPool,
}

impl DescriptorPool {
    pub fn new(device: VkDevice, max_sets: u32, pool_sizes: &[VkDescriptorPoolSize]) -> Self {
        let descriptor_pool_create_info = VkDescriptorPoolCreateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            maxSets: max_sets,
            poolSizeCount: pool_sizes.len() as u32,
            pPoolSizes: pool_sizes.as_ptr(),
        };

        let mut handle = core::ptr::null_mut();
        let result = unsafe {
            vkCreateDescriptorPool(
                device,
                &descriptor_pool_create_info,
                core::ptr::null_mut(),
                &mut handle,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create descriptor pool. Error: {}", result);
        }

        Self { handle }
    }

    pub fn allocate_set(&self, device: VkDevice, layout: VkDescriptorSetLayout) -> VkDescriptorSet {
        let layouts = [layout];
        let allocate_info = VkDescriptorSetAllocateInfo {
            sType: VK_STRUCTURE_TYPE_DESCRIPTOR_SET_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            descriptorPool: self.handle,
            descriptorSetCount: 1,
            pSetLayouts: layouts.as_ptr(),
        };

        let mut set = core::ptr::null_mut();
        let result = unsafe { vkAllocateDescriptorSets(device, &allocate_info, &mut set) };

        if result != VK_SUCCESS {
            panic!("Failed to allocate descriptor set. Error: {}", result);
        }

        set
    }

    pub fn destroy(&mut self, device: VkDevice) {
        unsafe {
            vkDestroyDescriptorPool(device, self.handle, core::ptr::null());
        }
    }
}
