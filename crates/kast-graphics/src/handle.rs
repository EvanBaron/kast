#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BufferHandle {
    pub index: u32,
    pub generation: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TextureHandle {
    pub index: u32,
    pub generation: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PipelineHandle {
    pub index: u32,
    pub generation: u32,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SamplerHandle {
    pub index: u32,
    pub generation: u32,
}
