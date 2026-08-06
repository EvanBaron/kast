use raw_window_handle::HasDisplayHandle;
use vk_bindings::*;

use crate::backend::vulkan::{
    surface::get_required_extensions,
    utils::{self, QueueFamily},
};

/// A Vulkan instance handle with automatic cleanup.
/// The instance is the connection between the application and the Vulkan library.
/// It manages validation layers in debug builds and provides methods for physical
/// device selection.
pub struct VulkanInstance {
    pub(crate) handle: VkInstance,
}

impl VulkanInstance {
    /// Creates a new Vulkan instance with required surface extensions.
    ///
    /// # Arguments
    /// * `app_name` - The name of the application.
    /// * `window` - A window handle implementing HasDisplayHandle for platform detection.
    ///
    /// # Returns
    /// A new VulkanInstance or an error string if creation fails.
    pub fn new(app_name: &str, window: &impl HasDisplayHandle) -> Result<Self, String> {
        let app_name_c =
            std::ffi::CString::new(app_name).map_err(|e| format!("Invalid app name: {}", e))?;
        let engine_name_c = c"Kast Engine";

        let application_info = VkApplicationInfo {
            sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: core::ptr::null(),
            pApplicationName: app_name_c.as_ptr(),
            applicationVersion: make_version(0, 0, 1, 0),
            pEngineName: engine_name_c.as_ptr(),
            engineVersion: make_version(0, 0, 1, 0),
            apiVersion: make_version(0, 1, 2, 0),
        };

        let mut extensions = get_required_extensions(window);
        extensions.extend_from_slice(&[
            VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME.as_ptr() as *const i8,
            VK_EXT_SURFACE_MAINTENANCE_1_EXTENSION_NAME.as_ptr() as *const i8,
            VK_KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME.as_ptr() as *const i8,
        ]);

        #[cfg(debug_assertions)]
        let layers = Self::create_validation_layer();
        #[cfg(not(debug_assertions))]
        let layers = vec![];

        let create_info = VkInstanceCreateInfo {
            sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0,
            pApplicationInfo: &application_info,
            enabledExtensionCount: extensions.len() as u32,
            ppEnabledExtensionNames: extensions.as_ptr(),
            enabledLayerCount: layers.len() as u32,
            ppEnabledLayerNames: layers.as_ptr(),
        };

        let mut handle = core::ptr::null_mut();

        unsafe {
            let result = vk_bindings::vkCreateInstance(&create_info, std::ptr::null(), &mut handle);

            if result != vk_bindings::VK_SUCCESS {
                return Err(format!("Failed to create Vulkan instance: {}", result));
            }
        };

        Ok(VulkanInstance { handle })
    }

    /// Creates and returns enabled validation layers for debug builds.
    ///
    /// Checks for available validation layers and enables the Khronos validation layer
    /// if present. Prints a message if requested layers are not available.
    fn create_validation_layer() -> Vec<*const i8> {
        let std_validation_layer = b"VK_LAYER_KHRONOS_validation\0";
        let layers = [std_validation_layer.as_ptr() as *const i8];

        let mut available_layer_count = 0;
        let mut available_layers = Vec::new();
        unsafe {
            vkEnumerateInstanceLayerProperties(&mut available_layer_count, core::ptr::null_mut());
        }

        available_layers.resize(available_layer_count as usize, VkLayerProperties::default());
        unsafe {
            vkEnumerateInstanceLayerProperties(
                &mut available_layer_count,
                available_layers.as_mut_ptr(),
            );
        }

        let mut enabled_layers = Vec::new();

        for layer in layers.iter() {
            let layer_name = unsafe { core::ffi::CStr::from_ptr(*layer) };

            let found = available_layers.iter().find(|available_layer| {
                let available_layer_name =
                    unsafe { core::ffi::CStr::from_ptr(available_layer.layerName.as_ptr()) };

                layer_name == available_layer_name
            });

            if found.is_none() {
                println!("Layer {:?} is not supported.", layer_name);
            } else {
                enabled_layers.push(*layer);
            }
        }

        enabled_layers
    }

    /// Selects the best available physical device based on a scoring system.
    ///
    /// Enumerates all physical devices and scores them based on:
    /// - Device type (discrete GPUs are preferred)
    /// - Maximum supported image dimensions
    /// - Required extensions support
    /// - Queue family availability (graphics + present)
    /// - Format feature support
    ///
    /// # Arguments
    /// * `surface` - The window surface for checking presentation support.
    ///
    /// # Returns
    /// A tuple of (physical device, graphics queue family, present queue family) or an error.
    pub fn pick_physical_device(
        &self,
        surface: VkSurfaceKHR,
    ) -> Result<(VkPhysicalDevice, QueueFamily, QueueFamily), String> {
        let mut count = 0;

        unsafe {
            let result = vkEnumeratePhysicalDevices(self.handle, &mut count, core::ptr::null_mut());
            if result != VK_SUCCESS {
                return Err(format!(
                    "Failed to enumerate physical devices. Error: {:?}",
                    result
                ));
            }
        };

        if count == 0 {
            return Err("No Vulkan supported devices found".to_string());
        }

        let mut devices = vec![core::ptr::null_mut(); count as usize];
        unsafe {
            let result = vkEnumeratePhysicalDevices(self.handle, &mut count, devices.as_mut_ptr());
            if result != VK_SUCCESS {
                return Err(format!(
                    "Failed to enumerate physical devices. Error: {:?}",
                    result
                ));
            }
        };

        let mut candidates = Vec::new();

        for &device in devices.iter() {
            if utils::check_device_extension_support(device) {
                if let Ok((graphics_family, present_family)) =
                    utils::find_queue_families(device, surface)
                {
                    // Score system to pick the best device
                    let mut score = 0;
                    let mut properties = VkPhysicalDeviceProperties::default();
                    unsafe {
                        vkGetPhysicalDeviceProperties(device, &mut properties);
                    }

                    if properties.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU {
                        score += 1000;
                    }

                    score += properties.limits.maxImageDimension2D;

                    let mut format_properties = VkFormatProperties::default();
                    unsafe {
                        vkGetPhysicalDeviceFormatProperties(
                            device,
                            VK_FORMAT_R8G8B8A8_SRGB,
                            &mut format_properties,
                        );
                    }

                    if format_properties.optimalTilingFeatures & VK_FORMAT_FEATURE_SAMPLED_IMAGE_BIT
                        != 0
                    {
                        candidates.push((score, device, graphics_family, present_family));
                    }
                }
            }
        }

        candidates.sort_by(|a, b| b.0.cmp(&a.0));

        if let Some((_, device, graphics, present)) = candidates.first() {
            Ok((*device, *graphics, *present))
        } else {
            Err("No suitable physical device found".to_string())
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        unsafe {
            vkDestroyInstance(self.handle, core::ptr::null());
        }
    }
}
