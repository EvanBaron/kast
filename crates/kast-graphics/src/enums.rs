pub enum BufferUsage {
    Vertex,
    Index,
    Uniform,
    Storage,
}

pub enum MemoryProperties {
    DeviceLocal,
    HostVisible,
    HostCoherent,
}

pub enum TextureFormat {
    Rgba8Srgb,
    Rgba8Unorm,
    Depth32,
}

pub enum FilterMode {
    Nearest,
    Linear,
}

pub enum AddressMode {
    Repeat,
    ClampToEdge,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PrimitiveTopology {
    #[default]
    TriangleList,
    TriangleStrip,
    LineList,
    PointList,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PolygonMode {
    #[default]
    Fill,
    Line,
    Point,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum CullMode {
    #[default]
    Back,
    Front,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum BlendMode {
    #[default]
    None,
    AlphaBlend,
    Additive,
}

#[derive(Debug)]
pub enum GraphicsError {
    InitializationFailed(String),
    OutOfMemory(String),
    DeviceLost,
    InvalidShader(String),
    Internal(String),
    Unknown(String),
}

impl core::fmt::Display for GraphicsError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl core::error::Error for GraphicsError {}
