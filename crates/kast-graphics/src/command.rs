use crate::handle::*;

pub struct DrawCall {
    pub pipeline: PipelineHandle,
    pub vertex_buffer: BufferHandle,
    pub index_buffer: Option<BufferHandle>,
    pub index_count: u32,
    pub instance_count: u32,
    pub push_constants: Option<Vec<u8>>,
}

pub struct DrawList {
    pub calls: Vec<DrawCall>,
}

impl DrawList {
    pub fn push(&mut self, call: DrawCall) {
        self.calls.push(call);
    }
    pub fn clear(&mut self) {
        self.calls.clear();
    }
}

pub struct ColorAttachment {
    pub target: Option<TextureHandle>,
    pub clear_color: Option<[f32; 4]>,
}

pub struct DepthAttachment {
    pub target: TextureHandle,
    pub clear_depth: Option<f32>,
}

pub struct RenderPassDescriptor {
    pub color_attachments: Vec<ColorAttachment>,
    pub depth_attachment: Option<DepthAttachment>,
}

pub struct RenderPass {
    pub descriptor: RenderPassDescriptor,
    pub draw_list: DrawList,
}

impl RenderPass {
    pub fn new(descriptor: RenderPassDescriptor) -> Self {
        Self {
            descriptor,
            draw_list: DrawList { calls: Vec::new() },
        }
    }
}
