use vk_bindings::*;

use crate::backend::vulkan::device::VulkanDevice;

/// A Vulkan command buffer for recording GPU commands.
///
/// Command buffers are used to record rendering and compute operations
/// that can be submitted to device queues for execution.
pub struct VulkanCommandBuffer {
    pub(crate) handle: VkCommandBuffer,
}

impl VulkanCommandBuffer {
    /// Begins recording commands into the command buffer.
    ///
    /// # Arguments
    /// * `one_time_submit` - If true, sets the ONE_TIME_SUBMIT flag indicating
    ///   this command buffer will be submitted only once before being reset.
    ///
    /// # Returns
    /// Ok(()) on success, or an error if beginning the command buffer fails.
    pub fn begin(&self, one_time_submit: bool) -> Result<(), String> {
        let mut flags = 0;
        if one_time_submit {
            flags |= VK_COMMAND_BUFFER_USAGE_ONE_TIME_SUBMIT_BIT;
        }

        let begin_info = VkCommandBufferBeginInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags,
            pInheritanceInfo: core::ptr::null(),
        };

        unsafe {
            let result = vkBeginCommandBuffer(self.handle, &begin_info);
            if result != VK_SUCCESS {
                return Err(format!("Failed to begin command buffer: {}", result));
            }
        }

        Ok(())
    }

    /// Ends recording commands into the command buffer.
    ///
    /// Must be called after begin() and all command recording is complete.
    ///
    /// # Returns
    /// Ok(()) on success, or an error if ending the command buffer fails.
    pub fn end(&self) -> Result<(), String> {
        unsafe {
            let result = vkEndCommandBuffer(self.handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to end command buffer: {}", result));
            }
        }

        Ok(())
    }

    /// Begins a render pass.
    pub fn begin_render_pass(
        &self,
        render_pass: VkRenderPass,
        framebuffer: VkFramebuffer,
        render_area: VkRect2D,
        clear_values: &[VkClearValue],
    ) {
        let render_pass_begin_info = VkRenderPassBeginInfo {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: core::ptr::null(),
            renderPass: render_pass,
            framebuffer,
            renderArea: render_area,
            clearValueCount: clear_values.len() as u32,
            pClearValues: clear_values.as_ptr(),
        };

        unsafe {
            vkCmdBeginRenderPass(
                self.handle,
                &render_pass_begin_info,
                VK_SUBPASS_CONTENTS_INLINE,
            );
        }
    }

    /// Ends the current render pass.
    pub fn end_render_pass(&self) {
        unsafe {
            vkCmdEndRenderPass(self.handle);
        }
    }

    /// Binds a graphics pipeline.
    pub fn bind_graphics_pipeline(&self, pipeline: VkPipeline) {
        unsafe {
            vkCmdBindPipeline(self.handle, VK_PIPELINE_BIND_POINT_GRAPHICS, pipeline);
        }
    }

    /// Binds a compute pipeline.
    pub fn bind_compute_pipeline(&self, pipeline: VkPipeline) {
        unsafe {
            vkCmdBindPipeline(self.handle, VK_PIPELINE_BIND_POINT_COMPUTE, pipeline);
        }
    }

    /// Sets the viewport.
    pub fn set_viewport(
        &self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    ) {
        let viewport = VkViewport {
            x,
            y,
            width,
            height,
            minDepth: min_depth,
            maxDepth: max_depth,
        };

        unsafe {
            vkCmdSetViewport(self.handle, 0, 1, &viewport);
        }
    }

    /// Sets the scissor rectangle.
    pub fn set_scissor(&self, x: i32, y: i32, width: u32, height: u32) {
        let scissor = VkRect2D {
            offset: VkOffset2D { x, y },
            extent: VkExtent2D { width, height },
        };

        unsafe {
            vkCmdSetScissor(self.handle, 0, 1, &scissor);
        }
    }

    /// Binds vertex buffers.
    pub fn bind_vertex_buffers(
        &self,
        first_binding: u32,
        buffers: &[VkBuffer],
        offsets: &[VkDeviceSize],
    ) {
        unsafe {
            vkCmdBindVertexBuffers(
                self.handle,
                first_binding,
                buffers.len() as u32,
                buffers.as_ptr(),
                offsets.as_ptr(),
            );
        }
    }

    /// Binds an index buffer.
    pub fn bind_index_buffer(
        &self,
        buffer: VkBuffer,
        offset: VkDeviceSize,
        index_type: VkIndexType,
    ) {
        unsafe {
            vkCmdBindIndexBuffer(self.handle, buffer, offset, index_type);
        }
    }

    /// Binds descriptor sets.
    pub fn bind_descriptor_sets(
        &self,
        pipeline_bind_point: VkPipelineBindPoint,
        pipeline_layout: VkPipelineLayout,
        first_set: u32,
        descriptor_sets: &[VkDescriptorSet],
        dynamic_offsets: &[u32],
    ) {
        unsafe {
            vkCmdBindDescriptorSets(
                self.handle,
                pipeline_bind_point,
                pipeline_layout,
                first_set,
                descriptor_sets.len() as u32,
                descriptor_sets.as_ptr(),
                dynamic_offsets.len() as u32,
                dynamic_offsets.as_ptr(),
            );
        }
    }

    /// Issues a draw command.
    pub fn draw(
        &self,
        vertex_count: u32,
        instance_count: u32,
        first_vertex: u32,
        first_instance: u32,
    ) {
        unsafe {
            vkCmdDraw(
                self.handle,
                vertex_count,
                instance_count,
                first_vertex,
                first_instance,
            );
        }
    }

    /// Issues an indexed draw command.
    pub fn draw_indexed(
        &self,
        index_count: u32,
        instance_count: u32,
        first_index: u32,
        vertex_offset: i32,
        first_instance: u32,
    ) {
        unsafe {
            vkCmdDrawIndexed(
                self.handle,
                index_count,
                instance_count,
                first_index,
                vertex_offset,
                first_instance,
            );
        }
    }

    /// Copies data from one buffer to another.
    pub fn copy_buffer(&self, src: VkBuffer, dst: VkBuffer, regions: &[VkBufferCopy]) {
        unsafe {
            vkCmdCopyBuffer(
                self.handle,
                src,
                dst,
                regions.len() as u32,
                regions.as_ptr(),
            );
        }
    }

    /// Copies data from a buffer to an image.
    pub fn copy_buffer_to_image(
        &self,
        buffer: VkBuffer,
        image: VkImage,
        layout: VkImageLayout,
        regions: &[VkBufferImageCopy],
    ) {
        unsafe {
            vkCmdCopyBufferToImage(
                self.handle,
                buffer,
                image,
                layout,
                regions.len() as u32,
                regions.as_ptr(),
            );
        }
    }

    /// Inserts a pipeline barrier for synchronization.
    pub fn pipeline_barrier(
        &self,
        src_stage_mask: VkPipelineStageFlags,
        dst_stage_mask: VkPipelineStageFlags,
        dependency_flags: VkDependencyFlags,
        memory_barriers: &[VkMemoryBarrier],
        buffer_barriers: &[VkBufferMemoryBarrier],
        image_barriers: &[VkImageMemoryBarrier],
    ) {
        unsafe {
            vkCmdPipelineBarrier(
                self.handle,
                src_stage_mask,
                dst_stage_mask,
                dependency_flags,
                memory_barriers.len() as u32,
                memory_barriers.as_ptr(),
                buffer_barriers.len() as u32,
                buffer_barriers.as_ptr(),
                image_barriers.len() as u32,
                image_barriers.as_ptr(),
            );
        }
    }

    /// Pushes constants to the pipeline.
    pub fn push_constants(
        &self,
        pipeline_layout: VkPipelineLayout,
        stage_flags: VkShaderStageFlags,
        offset: u32,
        size: u32,
        data: *const core::ffi::c_void,
    ) {
        unsafe {
            vkCmdPushConstants(
                self.handle,
                pipeline_layout,
                stage_flags,
                offset,
                size,
                data,
            );
        }
    }

    /// Dispatches a compute shader.
    pub fn dispatch(&self, group_count_x: u32, group_count_y: u32, group_count_z: u32) {
        unsafe {
            vkCmdDispatch(self.handle, group_count_x, group_count_y, group_count_z);
        }
    }
}

/// A Vulkan command pool for allocating command buffers.
///
/// Command pools manage the memory used by command buffers and can be
/// reset to reclaim all command buffers allocated from the pool.
pub struct VulkanCommandPool {
    pub(crate) handle: VkCommandPool,
    device: VkDevice,
}

impl VulkanCommandPool {
    /// Creates a new command pool for allocating command buffers.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device.
    /// * `queue_family_index` - The queue family that command buffers from this pool will be submitted to.
    /// * `allow_reset` - If true, command buffers can be individually reset; otherwise the entire pool must be reset.
    ///
    /// # Returns
    /// A new VulkanCommandPool or an error if creation fails.
    pub fn new(
        device: &VulkanDevice,
        queue_family_index: u32,
        allow_reset: bool,
    ) -> Result<Self, String> {
        let mut flags = 0;
        if allow_reset {
            flags |= VK_COMMAND_POOL_CREATE_RESET_COMMAND_BUFFER_BIT;
        }

        let create_info = VkCommandPoolCreateInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_POOL_CREATE_INFO,
            pNext: core::ptr::null(),
            flags,
            queueFamilyIndex: queue_family_index,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result =
                vkCreateCommandPool(device.handle, &create_info, core::ptr::null(), &mut handle);
            if result != VK_SUCCESS {
                return Err(format!("Failed to create command pool: {}", result));
            }
        }

        Ok(Self {
            handle,
            device: device.handle,
        })
    }

    /// Allocates command buffers from this command pool.
    ///
    /// # Arguments
    /// * `count` - The number of command buffers to allocate.
    ///
    /// # Returns
    /// A vector of VulkanCommandBuffer instances or an error if allocation fails.
    pub fn allocate_buffers(&self, count: u32) -> Result<Vec<VulkanCommandBuffer>, String> {
        let allocate_info = VkCommandBufferAllocateInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            commandPool: self.handle,
            level: VK_COMMAND_BUFFER_LEVEL_PRIMARY,
            commandBufferCount: count,
        };

        let mut handles = vec![core::ptr::null_mut(); count as usize];
        unsafe {
            let result =
                vkAllocateCommandBuffers(self.device, &allocate_info, handles.as_mut_ptr());
            if result != VK_SUCCESS {
                return Err(format!("Failed to allocate command buffers: {}", result));
            }
        }

        Ok(handles
            .into_iter()
            .map(|h| VulkanCommandBuffer { handle: h })
            .collect())
    }

    /// Resets the command pool, releasing all command buffers back to the pool.
    ///
    /// All command buffers allocated from this pool become invalid and must
    /// not be used after this call.
    pub fn reset(&self) {
        unsafe {
            vkResetCommandPool(self.device, self.handle, 0);
        }
    }
}

impl Drop for VulkanCommandPool {
    fn drop(&mut self) {
        unsafe {
            vkDestroyCommandPool(self.device, self.handle, core::ptr::null());
        }
    }
}
