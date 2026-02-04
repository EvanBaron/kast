use vk_bindings::*;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: [f32; 3],
    pub texture_coords: [f32; 2],
}

impl Vertex {
    pub fn get_binding_description() -> VkVertexInputBindingDescription {
        VkVertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<Vertex>() as u32,
            inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
        }
    }

    pub fn get_attribute_descriptions() -> [VkVertexInputAttributeDescription; 2] {
        [
            VkVertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: VK_FORMAT_R32G32B32_SFLOAT,
                offset: 0,
            },
            VkVertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: VK_FORMAT_R32G32_SFLOAT,
                offset: std::mem::size_of::<[f32; 3]>() as u32,
            },
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Mesh {
    pub index_count: u32,
    pub first_index: u32,
    pub vertex_offset: i32,
}

impl Mesh {
    pub fn new(index_count: u32, first_index: u32, vertex_offset: i32) -> Self {
        Self {
            index_count,
            first_index,
            vertex_offset,
        }
    }
}
