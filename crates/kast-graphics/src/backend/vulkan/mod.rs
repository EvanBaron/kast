use kast_resources::Pool;
use raw_window_handle::HasDisplayHandle;
use vk_bindings::*;

use crate::{
    backend::vulkan::{
        allocator::*, buffer::*, command::*, deletion_queue::*, descriptor::*, device::*, image::*,
        instance::*, pipeline::*, renderpass::*, sampler::*, shader::*, surface::*, swapchain::*,
        sync::*, upload_context::*,
    },
    *,
};

pub(crate) mod allocator;
pub(crate) mod buffer;
pub(crate) mod command;
pub(crate) mod deletion_queue;
pub(crate) mod descriptor;
pub(crate) mod device;
pub(crate) mod image;
pub(crate) mod instance;
pub(crate) mod pipeline;
pub(crate) mod renderpass;
pub(crate) mod sampler;
pub(crate) mod shader;
pub(crate) mod surface;
pub(crate) mod swapchain;
pub(crate) mod sync;
pub(crate) mod upload_context;
pub(crate) mod utils;

struct FrameData {
    command_buffer: VulkanCommandBuffer,
    image_available: VulkanSemaphore,
    in_flight: VulkanFence,
}

pub struct VulkanContext {
    buffers: Pool<VulkanBuffer>,
    textures: Pool<VulkanImage>,
    pipelines: Pool<VulkanPipeline>,
    samplers: Pool<VulkanSampler>,

    bindless_system: BindlessDescriptorSystem,

    frame_data: Vec<FrameData>,
    // Indexed by swapchain image index, NOT by frame-in-flight slot: acquired
    // image order isn't guaranteed to match the frame-in-flight round-robin, so
    // a semaphore signaled per-frame can still be pending presentation from a
    // previous use of the same image (VUID-vkQueueSubmit-pSignalSemaphores-00067).
    render_finished_semaphores: Vec<VulkanSemaphore>,
    command_pool: VulkanCommandPool,

    render_pass: VulkanRenderPass,
    swapchain: Option<VulkanSwapchain>,

    upload_context: UploadContext,
    deletion_queue: DeletionQueue,

    current_frame: usize,
    image_index: u32,

    allocator: VulkanAllocator,
    device: VulkanDevice,
    surface: VulkanSurface,
    instance: VulkanInstance,
}

impl GraphicsContext for VulkanContext {
    fn create_buffer(
        &mut self,
        descriptor: &BufferDescriptor,
    ) -> Result<BufferHandle, GraphicsError> {
        let vk_usage = match descriptor.usage {
            BufferUsage::Vertex => VK_BUFFER_USAGE_VERTEX_BUFFER_BIT,
            BufferUsage::Index => VK_BUFFER_USAGE_INDEX_BUFFER_BIT,
            BufferUsage::Uniform => VK_BUFFER_USAGE_UNIFORM_BUFFER_BIT,
            BufferUsage::Storage => VK_BUFFER_USAGE_STORAGE_BUFFER_BIT,
        };

        let vk_memory_flags = match descriptor.memory_properties {
            MemoryProperties::DeviceLocal => VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            MemoryProperties::HostVisible => VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT,
            MemoryProperties::HostCoherent => VK_MEMORY_PROPERTY_HOST_COHERENT_BIT,
        };

        let buffer = VulkanBuffer::new(
            &mut self.allocator,
            self.device.handle,
            descriptor.size,
            vk_usage,
            vk_memory_flags,
        )
        .map_err(|e| GraphicsError::OutOfMemory(e))?;

        let (index, generation) = self.buffers.insert(buffer);
        Ok(BufferHandle { index, generation })
    }

    fn create_texture(
        &mut self,
        descriptor: &TextureDescriptor,
    ) -> Result<TextureHandle, GraphicsError> {
        let vk_format = match descriptor.format {
            TextureFormat::Rgba8Srgb => VK_FORMAT_R8G8B8A8_SRGB,
            TextureFormat::Rgba8Unorm => VK_FORMAT_R8G8B8A8_UNORM,
            TextureFormat::Depth32 => VK_FORMAT_D32_SFLOAT,
        };

        let mut image = VulkanImage::new(
            &mut self.allocator,
            self.device.handle,
            descriptor.width,
            descriptor.height,
            vk_format,
            VK_IMAGE_TILING_OPTIMAL,
            VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT,
            VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT,
            descriptor.mip_levels,
        )
        .map_err(GraphicsError::OutOfMemory)?;

        let bindless_slot = self
            .bindless_system
            .allocate_texture_slot()
            .ok_or_else(|| {
                GraphicsError::OutOfMemory("Bindless texture slots exhausted".to_string())
            })?;

        self.bindless_system.update_texture(
            bindless_slot,
            image.view,
            VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
        );
        image.bindless_slot = Some(bindless_slot);

        let (index, generation) = self.textures.insert(image);
        Ok(TextureHandle { index, generation })
    }

    fn create_sampler(
        &mut self,
        descriptor: &SamplerDescriptor,
    ) -> Result<SamplerHandle, GraphicsError> {
        let vk_filter = match descriptor.filter {
            FilterMode::Nearest => VK_FILTER_NEAREST,
            FilterMode::Linear => VK_FILTER_LINEAR,
        };

        let vk_address_mode = match descriptor.address_mode {
            AddressMode::Repeat => VK_SAMPLER_ADDRESS_MODE_REPEAT,
            AddressMode::ClampToEdge => VK_SAMPLER_ADDRESS_MODE_CLAMP_TO_EDGE,
        };

        let mut sampler = VulkanSampler::new(
            &self.device,
            vk_filter,
            vk_filter,
            vk_address_mode,
            vk_address_mode,
            vk_address_mode,
            descriptor.anisotropy.is_some(),
            descriptor.anisotropy.unwrap_or(1.0),
        )
        .map_err(GraphicsError::OutOfMemory)?;

        let bindless_slot = self
            .bindless_system
            .allocate_sampler_slot()
            .ok_or_else(|| {
                GraphicsError::OutOfMemory("Bindless sampler slots exhausted".to_string())
            })?;

        self.bindless_system
            .update_sampler(bindless_slot, sampler.handle);
        sampler.bindless_slot = Some(bindless_slot);

        let (index, generation) = self.samplers.insert(sampler);
        Ok(SamplerHandle { index, generation })
    }

    fn create_pipeline(
        &mut self,
        descriptor: &PipelineDescriptor,
    ) -> Result<PipelineHandle, GraphicsError> {
        let vertex_shader = VulkanShader::new(&self.device, descriptor.vertex_shader)
            .map_err(GraphicsError::OutOfMemory)?;
        let fragment_shader = VulkanShader::new(&self.device, descriptor.fragment_shader)
            .map_err(GraphicsError::OutOfMemory)?;

        let entry_point = c"main";
        let shader_stages = [
            vertex_shader.create_stage_info(VK_SHADER_STAGE_VERTEX_BIT, entry_point),
            fragment_shader.create_stage_info(VK_SHADER_STAGE_FRAGMENT_BIT, entry_point),
        ];

        let pipeline_layout =
            VulkanPipelineLayout::new(&self.device, &[self.bindless_system.layout()], &[])
                .map_err(GraphicsError::OutOfMemory)?;

        let vk_color_blend_attachment = match descriptor.blend_mode {
            BlendMode::None => GraphicsPipelineConfig::default_color_blend_attachment(),
            BlendMode::AlphaBlend => GraphicsPipelineConfig::alpha_blend_attachment(),
            BlendMode::Additive => {
                let mut att = GraphicsPipelineConfig::alpha_blend_attachment();
                att.colorBlendOp = VK_BLEND_OP_ADD;
                att.srcColorBlendFactor = VK_BLEND_FACTOR_ONE;
                att.dstColorBlendFactor = VK_BLEND_FACTOR_ONE;
                att
            }
        };

        let vk_topology = match descriptor.topology {
            PrimitiveTopology::TriangleList => VK_PRIMITIVE_TOPOLOGY_TRIANGLE_LIST,
            PrimitiveTopology::TriangleStrip => VK_PRIMITIVE_TOPOLOGY_TRIANGLE_STRIP,
            PrimitiveTopology::LineList => VK_PRIMITIVE_TOPOLOGY_LINE_LIST,
            PrimitiveTopology::PointList => VK_PRIMITIVE_TOPOLOGY_POINT_LIST,
        };

        let vk_polygon_mode = match descriptor.polygon_mode {
            PolygonMode::Fill => VK_POLYGON_MODE_FILL,
            PolygonMode::Line => VK_POLYGON_MODE_LINE,
            PolygonMode::Point => VK_POLYGON_MODE_POINT,
        };

        let vk_cull_mode = match descriptor.cull_mode {
            CullMode::Back => VK_CULL_MODE_BACK_BIT,
            CullMode::Front => VK_CULL_MODE_FRONT_BIT,
            CullMode::None => VK_CULL_MODE_NONE,
        };

        // All attributes come from a single interleaved vertex buffer bound at
        // binding 0 (see `submit`, which only ever binds one vertex buffer per
        // draw call), so the stride is the sum of every attribute's size and
        // every attribute references binding 0 with its own byte offset.
        let vertex_stride: u32 = descriptor
            .vertex_layout
            .iter()
            .map(|attribute| attribute.format.size)
            .sum();

        let vertex_bindings = if descriptor.vertex_layout.is_empty() {
            Vec::new()
        } else {
            vec![VkVertexInputBindingDescription {
                binding: 0,
                stride: vertex_stride,
                inputRate: VK_VERTEX_INPUT_RATE_VERTEX,
            }]
        };

        let mut vertex_attributes = Vec::with_capacity(descriptor.vertex_layout.len());

        for (i, attribute) in descriptor.vertex_layout.iter().enumerate() {
            let vk_format = match attribute.format.components {
                1 => VK_FORMAT_R32_SFLOAT,
                2 => VK_FORMAT_R32G32_SFLOAT,
                3 => VK_FORMAT_R32G32B32_SFLOAT,
                4 => VK_FORMAT_R32G32B32A32_SFLOAT,
                _ => VK_FORMAT_R32G32B32A32_SFLOAT,
            };

            vertex_attributes.push(VkVertexInputAttributeDescription {
                location: i as u32,
                binding: 0,
                format: vk_format,
                offset: attribute.offset,
            });
        }

        let dynamic_states = [VK_DYNAMIC_STATE_VIEWPORT, VK_DYNAMIC_STATE_SCISSOR];
        let swapchain_extent = self
            .swapchain
            .as_ref()
            .map(|s| s.extent)
            .unwrap_or(VkExtent2D {
                width: 1,
                height: 1,
            });

        let config = GraphicsPipelineConfig {
            shader_stages: &shader_stages,
            vertex_bindings: &vertex_bindings,
            vertex_attributes: &vertex_attributes,
            topology: vk_topology,
            viewport_extent: swapchain_extent,
            polygon_mode: vk_polygon_mode,
            cull_mode: vk_cull_mode,
            front_face: VK_FRONT_FACE_COUNTER_CLOCKWISE,
            depth_test_enable: descriptor.depth_stencil.depth_test,
            depth_write_enable: descriptor.depth_stencil.depth_write,
            depth_compare_op: VK_COMPARE_OP_LESS,
            color_blend_attachments: &[vk_color_blend_attachment],
            dynamic_states: &dynamic_states,
            pipeline_layout: pipeline_layout.handle,
            render_pass: self.render_pass.handle,
            subpass: 0,
        };

        let pipeline = VulkanPipeline::new_graphics(&self.device, &config, pipeline_layout.handle)
            .map_err(GraphicsError::OutOfMemory)?;

        core::mem::forget(pipeline_layout);

        let (index, generation) = self.pipelines.insert(pipeline);
        Ok(PipelineHandle { index, generation })
    }

    fn upload_buffer(
        &mut self,
        handle: BufferHandle,
        data: &[u8],
    ) -> Result<(), crate::GraphicsError> {
        let VulkanContext {
            buffers, allocator, ..
        } = self;

        if let Some(buffer) = buffers.get(handle.index, handle.generation) {
            buffer
                .write_data(allocator, data)
                .map_err(GraphicsError::Unknown)
        } else {
            Err(GraphicsError::Unknown("Buffer not found".to_string()))
        }
    }

    fn upload_texture(&mut self, handle: TextureHandle, data: &[u8]) -> Result<(), GraphicsError> {
        let texture = self
            .textures
            .get(handle.index, handle.generation)
            .ok_or_else(|| GraphicsError::Unknown("Texture not found".to_string()))?;

        let command_raw = self
            .upload_context
            .begin()
            .map_err(GraphicsError::Unknown)?;
        let command = VulkanCommandBuffer {
            handle: command_raw,
        };

        texture.transition_layout(
            &command,
            VK_IMAGE_LAYOUT_UNDEFINED,
            VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
        );

        if let Some(texture) = self.textures.get(handle.index, handle.generation) {
            let staging_index = {
                self.upload_context
                    .get_staging_buffer_index(&mut self.allocator, data.len() as u64)
                    .map_err(GraphicsError::Unknown)?
            };

            let current_frame = self.upload_context.current_frame();

            let staging_handle = {
                let entry = &mut self.upload_context.staging_entries[staging_index];
                entry
                    .buffer
                    .write_data(&mut self.allocator, data)
                    .map_err(GraphicsError::Unknown)?;
                entry.frame_index = current_frame;
                entry.buffer.handle
            };

            texture.copy_from_buffer(&command, staging_handle, texture.width, texture.height);

            texture.transition_layout(
                &command,
                VK_IMAGE_LAYOUT_TRANSFER_DST_OPTIMAL,
                VK_IMAGE_LAYOUT_SHADER_READ_ONLY_OPTIMAL,
            );
        }

        if self.textures.get(handle.index, handle.generation).is_none() {
            return Err(GraphicsError::Unknown("Texture not found".to_string()));
        }

        self.upload_context
            .submit(self.device.graphics_queue)
            .map_err(GraphicsError::Unknown)?;

        Ok(())
    }

    fn destroy_buffer(&mut self, handle: BufferHandle) {
        if let Some(buffer) = self.buffers.remove(handle.index, handle.generation) {
            buffer.destroy(&mut self.deletion_queue);
        }
    }

    fn destroy_texture(&mut self, handle: TextureHandle) {
        if let Some(texture) = self.textures.remove(handle.index, handle.generation) {
            if let Some(slot) = texture.bindless_slot {
                self.bindless_system.free_texture_slot(slot);
            }
            texture.destroy(&mut self.deletion_queue);
        }
    }

    fn destroy_sampler(&mut self, handle: SamplerHandle) {
        if let Some(sampler) = self.samplers.remove(handle.index, handle.generation) {
            if let Some(slot) = sampler.bindless_slot {
                self.bindless_system.free_sampler_slot(slot);
            }
        }
    }

    fn destroy_pipeline(&mut self, handle: PipelineHandle) {
        self.pipelines.remove(handle.index, handle.generation);
    }

    fn begin_frame(&mut self) -> Result<(), GraphicsError> {
        let frame = &self.frame_data[self.current_frame];

        frame
            .in_flight
            .wait(u64::MAX)
            .map_err(GraphicsError::Internal)?;

        self.upload_context.next_frame();
        self.deletion_queue.next_frame(&mut self.allocator);

        let swapchain = self
            .swapchain
            .as_ref()
            .ok_or_else(|| GraphicsError::Internal("Swapchain is missing".to_string()))?;

        let mut image_index = 0;
        let result = unsafe {
            vkAcquireNextImageKHR(
                self.device.handle,
                swapchain.handle,
                u64::MAX,
                frame.image_available.handle,
                core::ptr::null_mut(),
                &mut image_index,
            )
        };

        if result == VK_ERROR_OUT_OF_DATE_KHR {
            return Err(GraphicsError::DeviceLost);
        } else if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            return Err(GraphicsError::Internal(format!(
                "Failed to acquire image: {}",
                result
            )));
        }

        self.image_index = image_index;

        frame.in_flight.reset().map_err(GraphicsError::Internal)?;
        frame
            .command_buffer
            .begin(true)
            .map_err(GraphicsError::Internal)?;

        Ok(())
    }

    fn submit(&mut self, passes: &[RenderPass]) -> Result<(), GraphicsError> {
        let frame = &self.frame_data[self.current_frame];
        let command_buffer = &frame.command_buffer;
        let swapchain = self.swapchain.as_ref().unwrap();

        let framebuffer = swapchain.framebuffers[self.image_index as usize];
        let render_area = VkRect2D {
            offset: VkOffset2D { x: 0, y: 0 },
            extent: swapchain.extent,
        };

        for pass in passes {
            let mut clear_values = Vec::new();
            for color_attachment in &pass.descriptor.color_attachments {
                let color = color_attachment.clear_color.unwrap_or([0.0, 0.0, 0.0, 1.0]);
                clear_values.push(VkClearValue {
                    color: VkClearColorValue { float32: color },
                });
            }

            if let Some(depth_attachment) = &pass.descriptor.depth_attachment {
                let depth = depth_attachment.clear_depth.unwrap_or(1.0);
                clear_values.push(VkClearValue {
                    depthStencil: VkClearDepthStencilValue { depth, stencil: 0 },
                });
            }

            command_buffer.begin_render_pass(
                self.render_pass.handle,
                framebuffer,
                render_area,
                &clear_values,
            );

            command_buffer.set_viewport(
                0.0,
                0.0,
                swapchain.extent.width as f32,
                swapchain.extent.height as f32,
                0.0,
                1.0,
            );

            command_buffer.set_scissor(0, 0, swapchain.extent.width, swapchain.extent.height);

            for call in &pass.draw_list.calls {
                if let Some(pipeline) = self
                    .pipelines
                    .get(call.pipeline.index, call.pipeline.generation)
                {
                    if pipeline.bind_point == VK_PIPELINE_BIND_POINT_GRAPHICS {
                        command_buffer.bind_graphics_pipeline(pipeline.handle);

                        let descriptor_sets = [self.bindless_system.descriptor_set()];
                        command_buffer.bind_descriptor_sets(
                            VK_PIPELINE_BIND_POINT_GRAPHICS,
                            pipeline.layout,
                            0,
                            &descriptor_sets,
                            &[],
                        );

                        if let Some(push_constants) = &call.push_constants {
                            command_buffer.push_constants(
                                pipeline.layout,
                                VK_SHADER_STAGE_ALL as u32,
                                0,
                                push_constants.len() as u32,
                                push_constants.as_ptr() as *const _,
                            );
                        }

                        if let Some(vertex_buffer) = self
                            .buffers
                            .get(call.vertex_buffer.index, call.vertex_buffer.generation)
                        {
                            command_buffer.bind_vertex_buffers(0, &[vertex_buffer.handle], &[0]);
                        }

                        if let Some(index_buffer) = call.index_buffer {
                            if let Some(ib) = self
                                .buffers
                                .get(index_buffer.index, index_buffer.generation)
                            {
                                command_buffer.bind_index_buffer(
                                    ib.handle,
                                    0,
                                    VK_INDEX_TYPE_UINT32,
                                );
                                command_buffer.draw_indexed(
                                    call.index_count,
                                    call.instance_count,
                                    0,
                                    0,
                                    0,
                                );
                            }
                        } else {
                            command_buffer.draw(call.index_count, call.instance_count, 0, 0);
                        }
                    }
                }
            }

            command_buffer.end_render_pass();
        }

        Ok(())
    }

    fn end_frame(&mut self) -> Result<(), GraphicsError> {
        let frame = &self.frame_data[self.current_frame];
        let command_buffer = &frame.command_buffer;

        command_buffer.end().map_err(GraphicsError::Internal)?;

        let wait_semaphores = [frame.image_available.handle];
        let wait_stages = [VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT];
        let signal_semaphores = [self.render_finished_semaphores[self.image_index as usize].handle];
        let command_buffers = [command_buffer.handle];

        let submit_info = VkSubmitInfo {
            sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: core::ptr::null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: wait_semaphores.as_ptr(),
            pWaitDstStageMask: wait_stages.as_ptr(),
            commandBufferCount: 1,
            pCommandBuffers: command_buffers.as_ptr(),
            signalSemaphoreCount: 1,
            pSignalSemaphores: signal_semaphores.as_ptr(),
        };

        unsafe {
            let result = vkQueueSubmit(
                self.device.graphics_queue,
                1,
                &submit_info,
                frame.in_flight.handle,
            );
            if result != VK_SUCCESS {
                return Err(GraphicsError::Internal(format!(
                    "Failed to submit queue: {}",
                    result
                )));
            }
        }

        let swapchains = [self.swapchain.as_ref().unwrap().handle];
        let image_indices = [self.image_index];

        let present_info = VkPresentInfoKHR {
            sType: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
            pNext: core::ptr::null(),
            waitSemaphoreCount: 1,
            pWaitSemaphores: signal_semaphores.as_ptr(),
            swapchainCount: 1,
            pSwapchains: swapchains.as_ptr(),
            pImageIndices: image_indices.as_ptr(),
            pResults: core::ptr::null_mut(),
        };

        let result = unsafe { vkQueuePresentKHR(self.device.present_queue, &present_info) };

        // VK_SUBOPTIMAL_KHR is a success code (the image still presents) — it's
        // just advisory that the swapchain no longer matches the surface exactly
        // and should be recreated soon. Only VK_ERROR_OUT_OF_DATE_KHR is a real
        // failure that requires recreating before presenting again.
        if result == VK_ERROR_OUT_OF_DATE_KHR {
            return Err(GraphicsError::DeviceLost);
        } else if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            return Err(GraphicsError::Internal(format!(
                "Failed to present image: {}",
                result
            )));
        }

        self.current_frame = (self.current_frame + 1) % self.frame_data.len();
        Ok(())
    }

    fn resize(&mut self, width: u32, height: u32) {
        let old = self.swapchain.take();

        match VulkanSwapchain::new(
            &self.device,
            self.surface.handle,
            width,
            height,
            old.as_ref(),
        ) {
            Ok(new_swapchain) => {
                // Image count can change across recreation, so the per-image
                // semaphore array is rebuilt to match (see the field's doc comment
                // for why it's per-image rather than per-frame-in-flight).
                match (0..new_swapchain.image_views.len())
                    .map(|_| VulkanSemaphore::new(self.device.handle))
                    .collect::<Result<Vec<_>, _>>()
                {
                    Ok(semaphores) => self.render_finished_semaphores = semaphores,
                    Err(e) => eprintln!("Failed to recreate render_finished semaphores: {}", e),
                }

                self.swapchain = Some(new_swapchain);
                if let Some(swapchain) = self.swapchain.as_mut() {
                    if let Err(e) = swapchain.create_framebuffers(self.render_pass.handle) {
                        eprintln!("Failed to recreate framebuffers on resize: {}", e);
                    }
                }
            }
            Err(e) => {
                eprintln!("Swapchain recreation failed: {}", e);
                self.swapchain = old;
            }
        }
    }
}

impl VulkanContext {
    pub fn new(
        app_name: &str,
        window: &(impl HasWindowHandle + HasDisplayHandle),
        width: u32,
        height: u32,
    ) -> Result<Self, GraphicsError> {
        let instance = VulkanInstance::new(app_name, window)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let surface = VulkanSurface::new(instance.handle, window)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let (physical_device, graphics_family, present_family) = instance
            .pick_physical_device(surface.handle)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let device = VulkanDevice::new(physical_device, graphics_family, present_family)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let allocator = VulkanAllocator::new(device.handle, physical_device);

        let swapchain = VulkanSwapchain::new(&device, surface.handle, width, height, None)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        // The swapchain framebuffers only bind the color image view (see
        // VulkanSwapchain::create_framebuffers), so declaring a depth attachment
        // here would make vkCreateFramebuffer fail on an attachment count mismatch.
        let render_pass = VulkanRenderPass::new(
            &device,
            swapchain.format.format,
            None,
            vk_bindings::VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        )
        .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let frames_in_flight = 3;
        let command_pool = VulkanCommandPool::new(&device, graphics_family.family_index, true)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let mut frame_data = Vec::with_capacity(frames_in_flight);
        let command_buffers = command_pool
            .allocate_buffers(frames_in_flight as u32)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        for command_buffer in command_buffers {
            frame_data.push(FrameData {
                command_buffer,
                image_available: VulkanSemaphore::new(device.handle)
                    .map_err(|e| GraphicsError::InitializationFailed(e))?,
                in_flight: VulkanFence::new(device.handle, true)
                    .map_err(|e| GraphicsError::InitializationFailed(e))?,
            });
        }

        let mut render_finished_semaphores = Vec::with_capacity(swapchain.image_views.len());
        for _ in 0..swapchain.image_views.len() {
            render_finished_semaphores.push(
                VulkanSemaphore::new(device.handle)
                    .map_err(|e| GraphicsError::InitializationFailed(e))?,
            );
        }

        let deletion_queue = DeletionQueue::new(frames_in_flight);
        let upload_context = UploadContext::new(
            device.handle,
            graphics_family.family_index,
            frames_in_flight,
        )
        .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let bindless_system =
            BindlessDescriptorSystem::new(&device, BindlessDescriptorConfig::default())
                .map_err(|e| GraphicsError::InitializationFailed(e))?;

        let mut context = Self {
            buffers: Pool::new(),
            textures: Pool::new(),
            pipelines: Pool::new(),
            samplers: Pool::new(),
            bindless_system,
            frame_data,
            render_finished_semaphores,
            command_pool,
            render_pass,
            swapchain: Some(swapchain),
            upload_context,
            deletion_queue,
            current_frame: 0,
            image_index: 0,
            allocator,
            device,
            surface,
            instance,
        };

        context
            .swapchain
            .as_mut()
            .unwrap()
            .create_framebuffers(context.render_pass.handle)
            .map_err(|e| GraphicsError::InitializationFailed(e))?;

        Ok(context)
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        self.device.wait_idle();

        for (_, buffer) in self.buffers.drain() {
            buffer.destroy(&mut self.deletion_queue);
        }
        for (_, texture) in self.textures.drain() {
            if let Some(slot) = texture.bindless_slot {
                self.bindless_system.free_texture_slot(slot);
            }
            texture.destroy(&mut self.deletion_queue);
        }

        self.deletion_queue.flush_all(&mut self.allocator);
        self.upload_context.destroy(&mut self.allocator);
    }
}
