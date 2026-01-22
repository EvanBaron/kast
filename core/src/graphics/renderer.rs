use crate::graphics::frame::FrameData;
use crate::graphics::instance::Instance;
use crate::graphics::mesh::{Mesh, Vertex};
use crate::graphics::pipeline::Pipeline;
use crate::graphics::swapchain::Swapchain;
use vk_bindings::*;
use winit::window::Window;

const MAX_FRAMES_IN_FLIGHT: usize = 3;

pub struct Renderer {
    pub swapchain: Swapchain,
    pub render_pass: VkRenderPass,
    pub pipeline: Pipeline,
    pub frames: Vec<FrameData>,
    pub current_frame_index: usize,
    pub mesh: Mesh,
    device: VkDevice,
    graphics_queue: VkQueue,
    present_queue: VkQueue,
}

impl Renderer {
    /// Creates a new renderer instance.
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance to create the renderer on.
    /// * `window` - The window to create the renderer for.
    pub fn new(instance: &Instance, window: &Window) -> Self {
        let mut swapchain = Swapchain::new(
            instance.physical_device,
            instance.device,
            instance.surface,
            instance.graphics_queue_family,
            instance.present_queue_family,
            window,
            core::ptr::null_mut(),
            None,
        );

        let render_pass = Self::create_renderpass(instance.device, swapchain.surface_format.format);

        let pipeline = Pipeline::new(instance.device, render_pass);

        swapchain.create_framebuffers(instance.device, render_pass);

        let mut frames = Vec::with_capacity(MAX_FRAMES_IN_FLIGHT);
        for _ in 0..MAX_FRAMES_IN_FLIGHT {
            frames.push(FrameData::new(
                instance.device,
                instance.graphics_queue_family,
            ));
        }

        let vertices: Vec<Vertex> = vec![
            Vertex {
                position: [-0.5, -0.5, 0.0],
            },
            Vertex {
                position: [0.5, -0.5, 0.0],
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
            },
        ];

        let indices = vec![0, 1, 2, 1, 3, 2];

        let mesh = Mesh::new(
            instance.device,
            instance.physical_device,
            &vertices,
            &indices,
        );

        Self {
            swapchain,
            render_pass,
            pipeline,
            frames,
            device: instance.device,
            current_frame_index: 0,
            mesh: mesh,
            graphics_queue: instance.graphics_queue,
            present_queue: instance.present_queue,
        }
    }

    /// Creates a renderpass with a single color attachment.
    ///
    /// # Arguments
    /// * `device` - The Vulkan device to create the renderpass on.
    /// * `format` - The format of the color attachment.
    fn create_renderpass(device: VkDevice, format: VkFormat) -> VkRenderPass {
        let mut attachment_descs = Vec::new();
        let attachment_descritpion = VkAttachmentDescription {
            flags: 0x0,
            format: format,
            samples: VK_SAMPLE_COUNT_1_BIT,
            // Clear the attachment at the start of the render pass
            loadOp: VK_ATTACHMENT_LOAD_OP_CLEAR,
            // Store the result so it can be presented
            storeOp: VK_ATTACHMENT_STORE_OP_STORE,
            stencilLoadOp: VK_ATTACHMENT_LOAD_OP_DONT_CARE,
            stencilStoreOp: VK_ATTACHMENT_STORE_OP_DONT_CARE,
            initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
            // Transition to PRESENT_SRC_KHR for presentation
            finalLayout: VK_IMAGE_LAYOUT_PRESENT_SRC_KHR,
        };

        attachment_descs.push(attachment_descritpion);

        let mut color_attachment_refs = Vec::new();
        let new_attachment_ref = VkAttachmentReference {
            attachment: 0,
            layout: VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
        };

        color_attachment_refs.push(new_attachment_ref);

        let mut subpass_descriptions = Vec::new();
        let subpass_description = VkSubpassDescription {
            flags: 0x0,
            pipelineBindPoint: VK_PIPELINE_BIND_POINT_GRAPHICS,
            inputAttachmentCount: 0,
            pInputAttachments: core::ptr::null(),
            colorAttachmentCount: color_attachment_refs.len() as u32,
            pColorAttachments: color_attachment_refs.as_ptr(),
            pResolveAttachments: core::ptr::null(),
            pDepthStencilAttachment: core::ptr::null(),
            preserveAttachmentCount: 0,
            pPreserveAttachments: core::ptr::null(),
        };

        subpass_descriptions.push(subpass_description);

        let mut subpass_dependencies = Vec::new();
        // Create a dependency to ensure the render pass doesn't start until the
        // image is available and ready for writing.
        let external_dependency = VkSubpassDependency {
            srcSubpass: VK_SUBPASS_EXTERNAL as u32,
            dstSubpass: 0,
            srcStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as VkPipelineStageFlags,
            dstStageMask: VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as VkPipelineStageFlags,
            srcAccessMask: 0x0,
            dstAccessMask: VK_ACCESS_COLOR_ATTACHMENT_WRITE_BIT as VkAccessFlags,
            dependencyFlags: 0x0,
        };

        subpass_dependencies.push(external_dependency);

        let render_pass_create_info = VkRenderPassCreateInfo {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            attachmentCount: attachment_descs.len() as u32,
            pAttachments: attachment_descs.as_ptr(),
            subpassCount: subpass_descriptions.len() as u32,
            pSubpasses: subpass_descriptions.as_ptr(),
            dependencyCount: subpass_dependencies.len() as u32,
            pDependencies: subpass_dependencies.as_ptr(),
        };

        println!("Creating render pass.");
        let mut render_pass = core::ptr::null_mut();
        let result = unsafe {
            vkCreateRenderPass(
                device,
                &render_pass_create_info,
                core::ptr::null_mut(),
                &mut render_pass,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create render pass. Error: {:?}.", result);
        }

        render_pass
    }

    pub fn draw_frame(&mut self, instance: &Instance, window: &Window) {
        self.current_frame_index = (self.current_frame_index + 1) % self.frames.len();

        // Wait for the previous frame to finish processing.
        let in_flight_fence = self.frames[self.current_frame_index].in_flight_fence;
        let present_fence = self.frames[self.current_frame_index].present_fence;

        let fences = [in_flight_fence, present_fence];

        let result =
            unsafe { vkWaitForFences(self.device, 2, fences.as_ptr(), VK_TRUE, core::u64::MAX) };

        if result != VK_SUCCESS {
            panic!("Error while waiting for fences. Error: {:?}.", result);
        }

        let frame = &mut self.frames[self.current_frame_index];
        unsafe {
            for &frame_buffer in &frame.delete_queue_framebuffers {
                vkDestroyFramebuffer(self.device, frame_buffer, core::ptr::null());
            }

            frame.delete_queue_framebuffers.clear();

            for &image_view in &frame.delete_queue_image_views {
                vkDestroyImageView(self.device, image_view, core::ptr::null());
            }

            frame.delete_queue_image_views.clear();
        }

        // Acquire the next image from the swapchain.
        let mut image_index: u32 = 0;
        let result = unsafe {
            vkAcquireNextImageKHR(
                self.device,
                self.swapchain.handle,
                core::u64::MAX,
                self.frames[self.current_frame_index].image_available_semaphore,
                core::ptr::null_mut(),
                &mut image_index,
            )
        };

        if result == VK_ERROR_OUT_OF_DATE_KHR {
            self.resize(instance, window);
            return;
        } else if result != VK_SUCCESS && result != VK_SUBOPTIMAL_KHR {
            panic!("Error while acquiring image: {:?}", result);
        }

        // Reset the fence to unsignaled state for the next frame.
        let result = unsafe { vkResetFences(self.device, 1, &in_flight_fence) };

        if result != VK_SUCCESS {
            panic!(
                "Error while resetting in-flight fence. Error: {:?}.",
                result
            );
        }

        let result = unsafe { vkResetFences(self.device, 1, &present_fence) };

        if result != VK_SUCCESS {
            panic!("Error while resetting present fence. Error: {:?}.", result);
        }

        // Reset the command pool to clear old commands.
        let result = unsafe {
            vkResetCommandPool(
                self.device,
                self.frames[self.current_frame_index].command_pool,
                0x0,
            )
        };

        if result != VK_SUCCESS {
            panic!("Error while resetting command pool. Error: {:?}.", result);
        }

        let command_buffer_begin_info = VkCommandBufferBeginInfo {
            sType: VK_STRUCTURE_TYPE_COMMAND_BUFFER_BEGIN_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            pInheritanceInfo: core::ptr::null(),
        };

        // Begin recording the command buffer.
        let result = unsafe {
            vkBeginCommandBuffer(
                self.frames[self.current_frame_index].command_buffer,
                &command_buffer_begin_info,
            )
        };

        if result != VK_SUCCESS {
            panic!(
                "Failed to start recording the comand buffer. Error: {:?}.",
                result
            );
        }

        let clear_value = [VkClearValue {
            color: VkClearColorValue {
                float32: [0.1, 0.2, 0.3, 1.0],
            },
        }];

        let render_pass_begin_info = VkRenderPassBeginInfo {
            sType: VK_STRUCTURE_TYPE_RENDER_PASS_BEGIN_INFO,
            pNext: core::ptr::null(),
            renderPass: self.render_pass,
            framebuffer: self.swapchain.framebuffers[image_index as usize],
            renderArea: VkRect2D {
                offset: VkOffset2D { x: 0, y: 0 },
                extent: self.swapchain.extent,
            },
            clearValueCount: clear_value.len() as u32,
            pClearValues: clear_value.as_ptr(),
        };

        unsafe {
            vkCmdBeginRenderPass(
                self.frames[self.current_frame_index].command_buffer,
                &render_pass_begin_info,
                VK_SUBPASS_CONTENTS_INLINE,
            );

            vkCmdBindPipeline(
                self.frames[self.current_frame_index].command_buffer,
                VK_PIPELINE_BIND_POINT_GRAPHICS,
                self.pipeline.handle,
            );

            let viewports = [VkViewport {
                x: 0.0,
                y: 0.0,
                width: self.swapchain.extent.width as f32,
                height: self.swapchain.extent.height as f32,
                minDepth: 0.0,
                maxDepth: 1.0,
            }];

            vkCmdSetViewport(
                self.frames[self.current_frame_index].command_buffer,
                0,
                viewports.len() as u32,
                viewports.as_ptr(),
            );

            let scissors = [VkRect2D {
                offset: VkOffset2D { x: 0, y: 0 },
                extent: VkExtent2D {
                    width: self.swapchain.extent.width,
                    height: self.swapchain.extent.height,
                },
            }];

            vkCmdSetScissor(
                self.frames[self.current_frame_index].command_buffer,
                0,
                scissors.len() as u32,
                scissors.as_ptr(),
            );

            let vertex_buffers = [self.mesh.vertex_buffer];
            let offsets = [0];

            vkCmdBindVertexBuffers(
                self.frames[self.current_frame_index].command_buffer,
                0,
                vertex_buffers.len() as u32,
                vertex_buffers.as_ptr(),
                offsets.as_ptr(),
            );

            vkCmdBindIndexBuffer(
                self.frames[self.current_frame_index].command_buffer,
                self.mesh.index_buffer,
                0,
                VK_INDEX_TYPE_UINT32,
            );

            vkCmdDrawIndexed(
                self.frames[self.current_frame_index].command_buffer,
                self.mesh.index_count,
                1,
                0,
                0,
                0,
            );

            vkCmdEndRenderPass(self.frames[self.current_frame_index].command_buffer);
        }

        let result =
            unsafe { vkEndCommandBuffer(self.frames[self.current_frame_index].command_buffer) };

        if result != VK_SUCCESS {
            panic!(
                "Failed to end recording the comand buffer. Error: {:?}.",
                result
            );
        }

        let wait_semaphores = [self.frames[self.current_frame_index].image_available_semaphore];
        let signal_semaphores = [self.frames[self.current_frame_index].render_finished_semaphore];
        let wait_pipeline_stages =
            [VK_PIPELINE_STAGE_COLOR_ATTACHMENT_OUTPUT_BIT as VkPipelineStageFlags];
        let command_buffer = [self.frames[self.current_frame_index].command_buffer];

        let submit_info = VkSubmitInfo {
            sType: VK_STRUCTURE_TYPE_SUBMIT_INFO,
            pNext: core::ptr::null(),
            waitSemaphoreCount: wait_semaphores.len() as u32,
            pWaitSemaphores: wait_semaphores.as_ptr(),
            pWaitDstStageMask: wait_pipeline_stages.as_ptr(),
            commandBufferCount: command_buffer.len() as u32,
            pCommandBuffers: command_buffer.as_ptr(),
            signalSemaphoreCount: signal_semaphores.len() as u32,
            pSignalSemaphores: signal_semaphores.as_ptr(),
        };

        // Submit the command buffer to the graphics queue.
        let result = unsafe {
            vkQueueSubmit(
                self.graphics_queue,
                1,
                &submit_info,
                self.frames[self.current_frame_index].in_flight_fence,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to submit rendering commands: {:?}.", result);
        }

        {
            let swapchains = [self.swapchain.handle];
            let image_indices = [image_index];
            let render_finished_semaphores =
                [self.frames[self.current_frame_index].render_finished_semaphore];

            let mut fences = [self.frames[self.current_frame_index].present_fence];
            let mut present_fence_info = VkSwapchainPresentFenceInfoEXT {
                sType: VK_STRUCTURE_TYPE_SWAPCHAIN_PRESENT_FENCE_INFO_EXT,
                pNext: core::ptr::null(),
                swapchainCount: swapchains.len() as u32,
                pFences: fences.as_mut_ptr(),
            };

            let present_info = VkPresentInfoKHR {
                sType: VK_STRUCTURE_TYPE_PRESENT_INFO_KHR,
                pNext: &mut present_fence_info as *mut _ as *mut core::ffi::c_void,
                waitSemaphoreCount: render_finished_semaphores.len() as u32,
                pWaitSemaphores: render_finished_semaphores.as_ptr(),
                swapchainCount: swapchains.len() as u32,
                pSwapchains: swapchains.as_ptr(),
                pImageIndices: image_indices.as_ptr(),
                pResults: core::ptr::null_mut(),
            };

            // Present the image to the screen.
            let result = unsafe { vkQueuePresentKHR(self.present_queue, &present_info) };

            if result == VK_ERROR_OUT_OF_DATE_KHR || result == VK_SUBOPTIMAL_KHR {
                self.resize(instance, window);
            } else if result != VK_SUCCESS {
                panic!("Error while submitting present: {:?}.", result);
            }
        }
    }

    /// Recreates the swapchain and the framebuffers.
    ///
    /// # Arguments
    /// * `instance` - The instance to query.
    /// * `window` - The window to query.
    pub fn resize(&mut self, instance: &Instance, window: &Window) {
        let frame = &mut self.frames[self.current_frame_index];
        frame
            .delete_queue_framebuffers
            .extend_from_slice(&self.swapchain.framebuffers);
        frame
            .delete_queue_image_views
            .extend_from_slice(&self.swapchain.image_views);

        let old_handle = self.swapchain.handle;
        self.swapchain = Swapchain::new(
            instance.physical_device,
            self.device,
            instance.surface,
            instance.graphics_queue_family,
            instance.present_queue_family,
            window,
            old_handle,
            Some(self.swapchain.surface_format),
        );

        self.swapchain
            .create_framebuffers(self.device, self.render_pass);
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let result = vkDeviceWaitIdle(self.device);
            if result != VK_SUCCESS {
                panic!(
                    "Error while waiting for device before cleanup. Error: {:?}.",
                    result
                );
            }

            for frame in self.frames.iter_mut() {
                frame.destroy(self.device);
            }

            self.mesh.destroy(self.device);
            self.pipeline.destroy();
            vkDestroyRenderPass(self.device, self.render_pass, core::ptr::null());
            self.swapchain.destroy();
        }
    }
}
