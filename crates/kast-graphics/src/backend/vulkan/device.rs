use vk_bindings::*;

use crate::backend::vulkan::utils::QueueFamily;

/// A Vulkan logical device with queue access.
///
/// Represents a logical connection to a physical device, managing device-level
/// resources and providing access to command queues for graphics and presentation.
pub struct VulkanDevice {
    pub(crate) handle: VkDevice,
    pub(crate) physical_device: VkPhysicalDevice,
    pub(crate) graphics_queue: VkQueue,
    pub(crate) present_queue: VkQueue,
    pub graphics_family: QueueFamily,
    pub present_family: QueueFamily,
}
impl VulkanDevice {
    /// Creates a new logical device with required extensions and features.
    ///
    /// # Arguments
    /// * `physical_device` - The physical device to create the logical device from.
    /// * `graphics_family` - The queue family index for graphics operations.
    /// * `present_family` - The queue family index for presentation operations.
    ///
    /// # Returns
    /// A new VulkanDevice with graphics and present queue handles, or an error.
    pub fn new(
        physical_device: VkPhysicalDevice,
        graphics_family: QueueFamily,
        present_family: QueueFamily,
    ) -> Result<Self, String> {
        let priority = [1.0];

        let mut queue_create_infos = Vec::new();

        // Use a set to handle cases where graphics and present families are the same
        let unique_families = if graphics_family.family_index == present_family.family_index {
            vec![graphics_family.family_index]
        } else {
            vec![graphics_family.family_index, present_family.family_index]
        };

        for family in unique_families {
            queue_create_infos.push(VkDeviceQueueCreateInfo {
                sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0,
                queueFamilyIndex: family,
                queueCount: 1,
                pQueuePriorities: priority.as_ptr(),
            });
        }

        let extensions = [
            VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const core::ffi::c_char,
            VK_EXT_SWAPCHAIN_MAINTENANCE_1_EXTENSION_NAME.as_ptr() as *const core::ffi::c_char,
        ];

        // Enable descriptor indexing features. The UpdateAfterBind bits must match
        // what the bindless descriptor set layout actually requests per-binding
        // (see BindlessDescriptorSystem) — leaving them off while the layout uses
        // VK_DESCRIPTOR_BINDING_UPDATE_AFTER_BIND_BIT is a validation error.
        let mut descriptor_indexing_features = VkPhysicalDeviceDescriptorIndexingFeatures {
            sType: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_DESCRIPTOR_INDEXING_FEATURES,
            pNext: core::ptr::null_mut(),
            descriptorBindingPartiallyBound: VK_TRUE,
            descriptorBindingUpdateUnusedWhilePending: VK_TRUE,
            runtimeDescriptorArray: VK_TRUE,
            descriptorBindingVariableDescriptorCount: VK_TRUE,
            descriptorBindingSampledImageUpdateAfterBind: VK_TRUE,
            descriptorBindingUniformBufferUpdateAfterBind: VK_TRUE,
            descriptorBindingStorageBufferUpdateAfterBind: VK_TRUE,
            ..Default::default()
        };

        // Base features
        let features = VkPhysicalDeviceFeatures {
            shaderSampledImageArrayDynamicIndexing: VK_TRUE,
            ..Default::default()
        };

        let create_info = VkDeviceCreateInfo {
            sType: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: &mut descriptor_indexing_features as *mut _ as *mut core::ffi::c_void,
            flags: 0,
            queueCreateInfoCount: queue_create_infos.len() as u32,
            pQueueCreateInfos: queue_create_infos.as_ptr(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: core::ptr::null(),
            enabledExtensionCount: extensions.len() as u32,
            ppEnabledExtensionNames: extensions.as_ptr(),
            pEnabledFeatures: &features,
        };

        let mut handle = core::ptr::null_mut();
        unsafe {
            let result = vkCreateDevice(
                physical_device,
                &create_info,
                core::ptr::null(),
                &mut handle,
            );
            if result != VK_SUCCESS {
                return Err(format!("Failed to create logical device: {}", result));
            }

            let mut graphics_queue = core::ptr::null_mut();
            vkGetDeviceQueue(handle, graphics_family.family_index, 0, &mut graphics_queue);

            let mut present_queue = core::ptr::null_mut();
            vkGetDeviceQueue(handle, present_family.family_index, 0, &mut present_queue);

            Ok(Self {
                handle,
                physical_device,
                graphics_queue,
                present_queue,
                graphics_family,
                present_family,
            })
        }
    }

    /// Waits for all device operations to complete.
    ///
    /// This is useful before destroying resources or during cleanup to ensure
    /// no operations are in flight.
    pub fn wait_idle(&self) {
        unsafe {
            vkDeviceWaitIdle(self.handle);
        }
    }
}

impl Drop for VulkanDevice {
    fn drop(&mut self) {
        unsafe {
            vkDeviceWaitIdle(self.handle);
            vkDestroyDevice(self.handle, core::ptr::null());
        }
    }
}
