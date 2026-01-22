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
