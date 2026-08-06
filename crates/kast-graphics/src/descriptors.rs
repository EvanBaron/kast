use crate::enums::*;

pub struct BufferDescriptor {
    pub size: u64,
    pub usage: BufferUsage,
    pub memory_properties: MemoryProperties,
}

pub struct TextureDescriptor {
    pub width: u32,
    pub height: u32,
    pub format: TextureFormat,
    pub mip_levels: u32,
}

pub struct SamplerDescriptor {
    pub filter: FilterMode,
    pub address_mode: AddressMode,
    pub anisotropy: Option<f32>,
}

pub struct VertexFormat {
    pub size: u32,
    pub components: u32,
}

pub struct VertexAttribute {
    pub format: VertexFormat,
    pub offset: u32,
}

#[derive(Clone, Default)]
pub struct DepthStencilState {
    pub depth_test: bool,
    pub depth_write: bool,
}

pub struct PipelineDescriptor<'a> {
    pub vertex_shader: &'a [u8],
    pub fragment_shader: &'a [u8],
    pub vertex_layout: &'a [VertexAttribute],
    pub topology: PrimitiveTopology,
    pub polygon_mode: PolygonMode,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    pub depth_stencil: DepthStencilState,
}

impl<'a> Default for PipelineDescriptor<'a> {
    fn default() -> Self {
        Self {
            vertex_shader: &[],
            fragment_shader: &[],
            vertex_layout: &[],
            topology: PrimitiveTopology::default(),
            polygon_mode: PolygonMode::default(),
            cull_mode: CullMode::default(),
            blend_mode: BlendMode::default(),
            depth_stencil: DepthStencilState::default(),
        }
    }
}
