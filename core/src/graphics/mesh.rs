use vk_bindings::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
}

impl Vertex {
    pub fn get_binding_description() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [VkVertexInputAttributeDescription; 1] {
        [VkVertexInputAttributeDescription {
            binding: 0,
            location: 0,
            format: VK_FORMAT_R32G32B32_SFLOAT,
            offset: 0,
        }]
    }
}

pub struct Mesh {
    pub vertex_buffer: VkBuffer,
    pub vertex_buffer_memory: VkDeviceMemory,
    pub vertex_count: u32,
    pub index_buffer: VkBuffer,
    pub index_buffer_memory: VkDeviceMemory,
    pub index_count: u32,
}

impl Mesh {
    /// Creates a new mesh.
    ///
    /// # Arguments
    /// * `device` - The device to create the mesh on.
    /// * `physical_device` - The physical device to query.
    /// * `vertices` - The vertices to create the mesh from.
    /// * `indices` - The indices to create the mesh from.
    pub fn new(
        device: VkDevice,
        physical_device: VkPhysicalDevice,
        vertices: &[Vertex],
        indices: &[u32],
    ) -> Self {
        // Vertex buffer creation and memory allocation
        let (vertex_buffer, vertex_buffer_memory) = create_buffer(
            device,
            physical_device,
            (vertices.len() * std::mem::size_of::<Vertex>()) as VkDeviceSize,
            VK_BUFFER_USAGE_VERTEX_BUFFER_BIT as VkBufferUsageFlags,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags
                | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT as VkMemoryPropertyFlags,
        );

        unsafe {
            let mut data = core::ptr::null_mut();
            let result = vkMapMemory(
                device,
                vertex_buffer_memory,
                0,
                (vertices.len() * std::mem::size_of::<Vertex>()) as VkDeviceSize,
                0,
                &mut data,
            );

            if result != VK_SUCCESS {
                panic!("Failed to map memory for vertex buffer. Error: {}.", result);
            }

            core::ptr::copy_nonoverlapping(vertices.as_ptr(), data as *mut Vertex, vertices.len());
            vkUnmapMemory(device, vertex_buffer_memory);
        }

        // Index buffer creation and memory allocation
        let (index_buffer, index_buffer_memory) = create_buffer(
            device,
            physical_device,
            (indices.len() * std::mem::size_of::<u32>()) as VkDeviceSize,
            VK_BUFFER_USAGE_INDEX_BUFFER_BIT as VkBufferUsageFlags,
            VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT as VkMemoryPropertyFlags
                | VK_MEMORY_PROPERTY_HOST_COHERENT_BIT as VkMemoryPropertyFlags,
        );

        unsafe {
            let mut data = std::ptr::null_mut();
            let result = vkMapMemory(
                device,
                index_buffer_memory,
                0,
                (indices.len() * std::mem::size_of::<u32>()) as VkDeviceSize,
                0,
                &mut data,
            );

            if result != VK_SUCCESS {
                panic!("Failed to map memory for index buffer. Error: {}.", result);
            }

            core::ptr::copy_nonoverlapping(indices.as_ptr(), data as *mut u32, indices.len());
            vkUnmapMemory(device, index_buffer_memory);
        }

        Self {
            vertex_buffer,
            vertex_buffer_memory,
            vertex_count: vertices.len() as u32,
            index_buffer,
            index_buffer_memory,
            index_count: indices.len() as u32,
        }
    }

    pub fn destroy(&self, device: VkDevice) {
        unsafe {
            vkDestroyBuffer(device, self.vertex_buffer, core::ptr::null());
            vkFreeMemory(device, self.vertex_buffer_memory, core::ptr::null());
            vkDestroyBuffer(device, self.index_buffer, core::ptr::null());
            vkFreeMemory(device, self.index_buffer_memory, core::ptr::null());
        }
    }
}

/// Helper function to create a Vulkan buffer and allocate memory for it.
///
/// # Arguments
/// * `device` - The Vulkan device.
/// * `physical_device` - The Vulkan physical device.
/// * `size` - The size of the buffer in bytes.
/// * `usage` - The usage flags for the buffer.
/// * `properties` - The memory properties for the buffer.
fn create_buffer(
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    size: VkDeviceSize,
    usage: VkBufferUsageFlags,
    properties: VkMemoryPropertyFlags,
) -> (VkBuffer, VkDeviceMemory) {
    let buffer_create_info = VkBufferCreateInfo {
        sType: VK_STRUCTURE_TYPE_BUFFER_CREATE_INFO,
        pNext: core::ptr::null(),
        flags: 0,
        size,
        usage,
        sharingMode: VK_SHARING_MODE_EXCLUSIVE,
        queueFamilyIndexCount: 0,
        pQueueFamilyIndices: core::ptr::null(),
    };

    let mut buffer = core::ptr::null_mut();
    let result =
        unsafe { vkCreateBuffer(device, &buffer_create_info, core::ptr::null(), &mut buffer) };

    if result != VK_SUCCESS {
        panic!("Failed to create buffer. Error: {}.", result);
    }

    let mut memory_requirements = VkMemoryRequirements::default();
    unsafe {
        vkGetBufferMemoryRequirements(device, buffer, &mut memory_requirements);
    };

    let mut memory_properties = VkPhysicalDeviceMemoryProperties::default();
    unsafe {
        vkGetPhysicalDeviceMemoryProperties(physical_device, &mut memory_properties);
    };

    let memory_type_index = (0..memory_properties.memoryTypeCount)
        .find(|&i| {
            (memory_requirements.memoryTypeBits & (1 << i)) != 0
                && (memory_properties.memoryTypes[i as usize].propertyFlags & properties)
                    == properties
        })
        .unwrap_or_else(|| panic!("Failed to find suitable memory type for buffer."));

    let memory_allocation_info = VkMemoryAllocateInfo {
        sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
        pNext: core::ptr::null(),
        allocationSize: memory_requirements.size,
        memoryTypeIndex: memory_type_index,
    };

    let mut buffer_memory = core::ptr::null_mut();
    let result = unsafe {
        vkAllocateMemory(
            device,
            &memory_allocation_info,
            core::ptr::null(),
            &mut buffer_memory,
        )
    };

    if result != VK_SUCCESS {
        panic!("Failed to allocate buffer memory. Error: {}.", result);
    }

    let result = unsafe { vkBindBufferMemory(device, buffer, buffer_memory, 0) };

    if result != VK_SUCCESS {
        panic!("Failed to bind buffer memory. Error: {}.", result);
    }

    (buffer, buffer_memory)
}
