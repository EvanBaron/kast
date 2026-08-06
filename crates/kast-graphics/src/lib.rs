use raw_window_handle::HasWindowHandle;

use crate::{command::*, descriptors::*, enums::*, handle::*};
pub use backend::vulkan::VulkanContext;

mod backend;
pub mod command;
pub mod descriptors;
pub mod enums;
pub mod handle;

pub trait GraphicsContext {
    fn create_buffer(
        &mut self,
        descriptor: &BufferDescriptor,
    ) -> Result<BufferHandle, GraphicsError>;
    fn create_texture(
        &mut self,
        descriptor: &TextureDescriptor,
    ) -> Result<TextureHandle, GraphicsError>;
    fn create_sampler(
        &mut self,
        descriptor: &SamplerDescriptor,
    ) -> Result<SamplerHandle, GraphicsError>;
    fn create_pipeline(
        &mut self,
        descriptor: &PipelineDescriptor,
    ) -> Result<PipelineHandle, GraphicsError>;

    fn upload_buffer(&mut self, handle: BufferHandle, data: &[u8]) -> Result<(), GraphicsError>;
    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<(), GraphicsError>;

    fn destroy_buffer(&mut self, handle: BufferHandle);
    fn destroy_texture(&mut self, handle: TextureHandle);
    fn destroy_sampler(&mut self, handle: SamplerHandle);
    fn destroy_pipeline(&mut self, handle: PipelineHandle);

    fn begin_frame(&mut self) -> Result<(), GraphicsError>;
    fn submit(&mut self, passes: &[RenderPass]) -> Result<(), GraphicsError>;
    fn end_frame(&mut self) -> Result<(), GraphicsError>;

    fn resize(&mut self, width: u32, height: u32);
}
