use crate::window;
use std::ffi::CStr;
use vk_bindings::*;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

const APPLICATION_NAME: &'static CStr = unsafe { CStr::from_bytes_with_nul_unchecked(b"kast\0") };

#[derive(Clone, Copy, Debug)]
pub struct QueueFamily {
    pub family_index: u32,
    pub queue_index: u32,
}

impl QueueFamily {
    pub fn new(family_index: u32, queue_index: u32) -> Self {
        Self {
            family_index,
            queue_index,
        }
    }
}

pub struct Instance {
    pub instance: VkInstance,
    pub surface: VkSurfaceKHR,
    pub device: VkDevice,
    pub physical_device: VkPhysicalDevice,
    pub graphics_queue_family: QueueFamily,
    pub present_queue_family: QueueFamily,
    pub graphics_queue: VkQueue,
    pub present_queue: VkQueue,
}

impl Instance {
    /// Creates a new Vulkan Instance.
    ///
    /// # Arguments
    /// * `event_loop` - Reference to the active event loop.
    /// * `window` - Reference to the winit window.
    pub fn new(event_loop: &ActiveEventLoop, window: &Window) -> Self {
        let instance = Self::init_instance(event_loop);
        let surface = Self::create_surface(instance, window);
        let (physical_device, graphics_queue_family, present_queue_family) =
            Self::pick_physical_device(instance, surface);

        let device =
            Self::create_device(physical_device, graphics_queue_family, present_queue_family);

        let mut graphics_queue = core::ptr::null_mut();
        unsafe {
            vkGetDeviceQueue(
                device,
                graphics_queue_family.family_index,
                graphics_queue_family.queue_index,
                &mut graphics_queue,
            );
        }

        let mut present_queue = core::ptr::null_mut();
        unsafe {
            vkGetDeviceQueue(
                device,
                present_queue_family.family_index,
                present_queue_family.queue_index,
                &mut present_queue,
            );
        }

        let mut physical_device_properties = VkPhysicalDeviceProperties::default();
        unsafe {
            vkGetPhysicalDeviceProperties(physical_device, &mut physical_device_properties);
        }

        let device_name =
            unsafe { core::ffi::CStr::from_ptr(physical_device_properties.deviceName.as_ptr()) };

        println!("Chosen device name: {}", device_name.to_str().unwrap());

        Self {
            instance,
            surface,
            device,
            physical_device,
            graphics_queue_family,
            present_queue_family,
            graphics_queue,
            present_queue,
        }
    }

    /// Create the validation layer for Vulkan.
    /// Only used in debug builds (`debug_assertions`) to validate the Vulkan API calls.
    /// This helps catch errors and misuse of the API during development.
    ///
    /// Returns a vector of pointers to the validation layer names.
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

    /// Initializes a new Vulkan instance.
    ///
    /// # Arguments
    /// * `event_loop` - Reference to the active event loop.
    fn init_instance(event_loop: &ActiveEventLoop) -> VkInstance {
        let mut instance_extensions = window::get_required_instance_extensions(event_loop);
        instance_extensions.push(
            VK_KHR_GET_PHYSICAL_DEVICE_PROPERTIES_2_EXTENSION_NAME.as_ptr()
                as *const core::ffi::c_char,
        );
        instance_extensions.push(
            VK_EXT_SURFACE_MAINTENANCE_1_EXTENSION_NAME.as_ptr() as *const core::ffi::c_char,
        );
        instance_extensions.push(
            VK_KHR_GET_SURFACE_CAPABILITIES_2_EXTENSION_NAME.as_ptr() as *const core::ffi::c_char,
        );

        #[cfg(debug_assertions)]
        let layers = Self::create_validation_layer();
        #[cfg(not(debug_assertions))]
        let layers = vec![];

        let application_info = VkApplicationInfo {
            sType: VK_STRUCTURE_TYPE_APPLICATION_INFO,
            pNext: core::ptr::null(),
            pApplicationName: APPLICATION_NAME.as_ptr(),
            applicationVersion: make_version(0, 0, 1, 0),
            pEngineName: APPLICATION_NAME.as_ptr(),
            engineVersion: make_version(0, 0, 1, 0),
            apiVersion: make_version(0, 1, 1, 0),
        };

        let create_info = VkInstanceCreateInfo {
            sType: VK_STRUCTURE_TYPE_INSTANCE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            pApplicationInfo: &application_info,
            enabledExtensionCount: instance_extensions.len() as u32,
            ppEnabledExtensionNames: instance_extensions.as_ptr(),
            enabledLayerCount: layers.len() as u32,
            ppEnabledLayerNames: layers.as_ptr(),
        };

        let mut instance = core::ptr::null_mut();
        let result = unsafe { vkCreateInstance(&create_info, core::ptr::null(), &mut instance) };
        if result != VK_SUCCESS {
            panic!("Failed to create Vulkan instance. Error: {:?}", result);
        }

        instance
    }

    /// Create the Vulkan surface for the given window.
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance to create the surface for.
    /// * `window` - The window to create the surface for.
    fn create_surface(instance: VkInstance, window: &winit::window::Window) -> VkSurfaceKHR {
        use winit::raw_window_handle::{
            HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle,
        };

        let mut surface = core::ptr::null_mut();

        let display_handle = window.display_handle().unwrap().as_raw();
        let window_handle = window.window_handle().unwrap().as_raw();

        let result = match (display_handle, window_handle) {
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(window)) => {
                let create_info = VkXlibSurfaceCreateInfoKHR {
                    sType: VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR,
                    pNext: core::ptr::null(),
                    flags: 0,
                    dpy: display
                        .display
                        .map(|ptr| ptr.as_ptr())
                        .unwrap_or(core::ptr::null_mut()) as *mut _,
                    window: window.window,
                };
                unsafe {
                    vkCreateXlibSurfaceKHR(instance, &create_info, core::ptr::null(), &mut surface)
                }
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            (RawDisplayHandle::Xcb(display), RawWindowHandle::Xcb(window)) => {
                let create_info = VkXcbSurfaceCreateInfoKHR {
                    sType: VK_STRUCTURE_TYPE_XCB_SURFACE_CREATE_INFO_KHR,
                    pNext: core::ptr::null(),
                    flags: 0,
                    connection: display
                        .connection
                        .map(|ptr| ptr.as_ptr())
                        .unwrap_or(core::ptr::null_mut()) as *mut _,
                    window: window.window.get(),
                };
                unsafe {
                    vkCreateXcbSurfaceKHR(instance, &create_info, core::ptr::null(), &mut surface)
                }
            }
            #[cfg(any(
                target_os = "linux",
                target_os = "freebsd",
                target_os = "dragonfly",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            (RawDisplayHandle::Wayland(display), RawWindowHandle::Wayland(window)) => {
                let create_info = VkWaylandSurfaceCreateInfoKHR {
                    sType: VK_STRUCTURE_TYPE_WAYLAND_SURFACE_CREATE_INFO_KHR,
                    pNext: core::ptr::null(),
                    flags: 0,
                    display: display.display.as_ptr() as *mut _,
                    surface: window.surface.as_ptr() as *mut _,
                };
                unsafe {
                    vkCreateWaylandSurfaceKHR(
                        instance,
                        &create_info,
                        core::ptr::null(),
                        &mut surface,
                    )
                }
            }
            #[cfg(target_os = "windows")]
            (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
                let create_info = VkWin32SurfaceCreateInfoKHR {
                    sType: VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
                    pNext: core::ptr::null(),
                    flags: 0,
                    hinstance: window
                        .hinstance
                        .map(|h| h.get() as *mut core::ffi::c_void)
                        .unwrap_or(core::ptr::null_mut()),
                    hwnd: window.hwnd.get() as *mut core::ffi::c_void,
                };
                unsafe {
                    vkCreateWin32SurfaceKHR(instance, &create_info, core::ptr::null(), &mut surface)
                }
            }
            _ => panic!("Unsupported platform for surface creation"),
        };

        if result != VK_SUCCESS {
            panic!("Failed to create surface. Error: {:?}", result);
        }

        surface
    }

    /// Find the queue families for graphics and presentation.
    ///
    /// # Arguments
    /// * `physical_device` - The physical device to query.
    /// * `surface` - The surface to query.
    ///
    /// # Returns
    /// A tuple containing:
    /// * `QueueFamily` - Contains the indices of the graphics queue and family.
    /// * `QueueFamily` - Contains the indices of the presentation queue and family.
    fn find_queue_families(
        physical_device: VkPhysicalDevice,
        surface: VkSurfaceKHR,
    ) -> Result<(QueueFamily, QueueFamily), &'static str> {
        let mut queue_family_count: u32 = 0;
        unsafe {
            vkGetPhysicalDeviceQueueFamilyProperties(
                physical_device,
                &mut queue_family_count,
                core::ptr::null_mut(),
            );
        }

        let mut queue_families =
            vec![VkQueueFamilyProperties::default(); queue_family_count as usize];
        unsafe {
            vkGetPhysicalDeviceQueueFamilyProperties(
                physical_device,
                &mut queue_family_count,
                queue_families.as_mut_ptr(),
            );
        }

        let mut graphics_queue_family: i32 = -1;
        let mut present_queue_family: i32 = -1;

        for (i, queue_family) in queue_families.iter().enumerate() {
            let queue_family_index = i as i32;

            let mut present_support: VkBool32 = VK_FALSE;
            let result = unsafe {
                vkGetPhysicalDeviceSurfaceSupportKHR(
                    physical_device,
                    queue_family_index as u32,
                    surface,
                    &mut present_support,
                )
            };

            let present_supported = result == VK_SUCCESS && present_support != VK_FALSE;

            // Only one queue family can support both graphics and presentation
            if queue_family.queueFlags & VK_QUEUE_GRAPHICS_BIT as VkQueueFlags != 0 {
                if present_supported {
                    return Ok((
                        QueueFamily::new(queue_family_index as u32, 0),
                        QueueFamily::new(queue_family_index as u32, 0),
                    ));
                }

                if graphics_queue_family == -1 {
                    graphics_queue_family = queue_family_index;
                }
            }

            if present_supported && present_queue_family == -1 {
                present_queue_family = queue_family_index;
            }
        }

        // 2 Queue families supporting graphics and presentation
        if graphics_queue_family != -1 && present_queue_family != -1 {
            Ok((
                QueueFamily::new(graphics_queue_family as u32, 0),
                QueueFamily::new(present_queue_family as u32, 0),
            ))
        } else {
            Err("Could not find suitable queue families")
        }
    }

    /// Helper function to check if the device supports the required extensions.
    ///
    /// # Arguments
    /// * `physical_device` - The physical device to check.
    ///
    /// # Returns
    /// * `bool` - `true` if the device supports the required extensions, `false` otherwise.
    fn check_device_extension_support(physical_device: VkPhysicalDevice) -> bool {
        let mut device_extensions_count: u32 = 0;
        unsafe {
            vkEnumerateDeviceExtensionProperties(
                physical_device,
                core::ptr::null(),
                &mut device_extensions_count,
                core::ptr::null_mut(),
            );
        }

        let mut device_extensions =
            vec![VkExtensionProperties::default(); device_extensions_count as usize];
        unsafe {
            vkEnumerateDeviceExtensionProperties(
                physical_device,
                core::ptr::null(),
                &mut device_extensions_count,
                device_extensions.as_mut_ptr(),
            );
        }

        let extension_name = unsafe {
            core::ffi::CStr::from_bytes_with_nul_unchecked(VK_KHR_SWAPCHAIN_EXTENSION_NAME)
        };

        device_extensions.iter().any(|extension_properties| {
            let current_extension_name =
                unsafe { core::ffi::CStr::from_ptr(extension_properties.extensionName.as_ptr()) };

            current_extension_name == extension_name
        })
    }

    /// Pick the physical device that supports the given surface.
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance to pick the physical device for.
    /// * `surface` - The Vulkan surface to pick the physical device for.
    ///
    /// # Returns
    /// A Tuple containing :
    /// * `VkPhysicalDevice` - the physical device.
    /// * `QueueFamily` - The graphics queue family.
    /// * `QueueFamily` - The presentation queue family.
    fn pick_physical_device(
        instance: VkInstance,
        surface: VkSurfaceKHR,
    ) -> (VkPhysicalDevice, QueueFamily, QueueFamily) {
        let mut physical_device_count: u32 = 0;
        let result = unsafe {
            vkEnumeratePhysicalDevices(instance, &mut physical_device_count, core::ptr::null_mut())
        };

        if result != VK_SUCCESS {
            panic!("Failed to enumerate physical devices. Error: {:?}", result);
        }

        if physical_device_count == 0 {
            panic!("No vulkan capable devices found");
        }

        let mut physical_devices = vec![core::ptr::null_mut(); physical_device_count as usize];
        let result = unsafe {
            vkEnumeratePhysicalDevices(
                instance,
                &mut physical_device_count,
                physical_devices.as_mut_ptr(),
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to enumerate physical devices. Error: {:?}", result);
        }

        let mut candidates = Vec::new();

        for &physical_device in physical_devices.iter() {
            if Self::check_device_extension_support(physical_device) {
                if let Ok((graphics_family, present_family)) =
                    Self::find_queue_families(physical_device, surface)
                {
                    // Score system to pick the best device
                    let mut score = 0;
                    let mut properties = VkPhysicalDeviceProperties::default();
                    unsafe {
                        vkGetPhysicalDeviceProperties(physical_device, &mut properties);
                    }

                    // Prioritize discrete GPUs as they generally offer better performance
                    if properties.deviceType == VK_PHYSICAL_DEVICE_TYPE_DISCRETE_GPU {
                        score += 1000;
                    }

                    // Also consider the maximum image dimension supported
                    score += properties.limits.maxImageDimension2D;

                    candidates.push((score, physical_device, graphics_family, present_family));
                }
            }
        }

        // Sort by score in descending order
        candidates.sort_by(|a, b| b.0.cmp(&a.0));

        if let Some((_, device, graphics, present)) = candidates.first() {
            let mut properties = VkPhysicalDeviceProperties::default();
            unsafe {
                vkGetPhysicalDeviceProperties(*device, &mut properties);
            }

            return (*device, *graphics, *present);
        }

        panic!("No suitable physical device found with surface support.");
    }

    /// Creates a new Vulkan device.
    ///
    /// # Arguments
    /// * `physical_device` - Reference to the physical device.
    /// * `graphics_queue_family` - Reference to the graphics queue family.
    /// * `present_queue_family` - Reference to the present queue family.
    fn create_device(
        physical_device: VkPhysicalDevice,
        graphics_queue_family: QueueFamily,
        present_queue_family: QueueFamily,
    ) -> VkDevice {
        let queue_priorities: [f32; 2] = [1.0; 2];

        let queue_create_info_count: u32;
        let mut queue_create_infos = [VkDeviceQueueCreateInfo::default(); 2];

        // Check if the graphics and presentation queue families are the same.
        // If they are, we only need to create one queue.
        // If they are different, we need to create two separate queues.
        if graphics_queue_family.family_index == present_queue_family.family_index {
            let family_queue_count;
            if graphics_queue_family.queue_index == present_queue_family.queue_index {
                family_queue_count = 1;
            } else {
                family_queue_count = 2;
            }

            queue_create_infos[0] = VkDeviceQueueCreateInfo {
                sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                queueFamilyIndex: graphics_queue_family.family_index,
                queueCount: family_queue_count,
                pQueuePriorities: queue_priorities.as_ptr(),
            };

            queue_create_info_count = 1;
        } else {
            queue_create_infos[0] = VkDeviceQueueCreateInfo {
                sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                queueFamilyIndex: graphics_queue_family.family_index,
                queueCount: 1,
                pQueuePriorities: queue_priorities.as_ptr(),
            };

            queue_create_infos[1] = VkDeviceQueueCreateInfo {
                sType: VK_STRUCTURE_TYPE_DEVICE_QUEUE_CREATE_INFO,
                pNext: core::ptr::null(),
                flags: 0x0,
                queueFamilyIndex: present_queue_family.family_index,
                queueCount: 1,
                pQueuePriorities: queue_priorities.as_ptr(),
            };

            queue_create_info_count = 2;
        }

        let device_extensions = [
            VK_KHR_SWAPCHAIN_EXTENSION_NAME.as_ptr() as *const core::ffi::c_char,
            VK_EXT_SWAPCHAIN_MAINTENANCE_1_EXTENSION_NAME.as_ptr() as *const core::ffi::c_char,
        ];
        let physical_device_features = VkPhysicalDeviceFeatures::default();

        let mut swapchain_maintenance1_features =
            VkPhysicalDeviceSwapchainMaintenance1FeaturesEXT {
                sType: VK_STRUCTURE_TYPE_PHYSICAL_DEVICE_SWAPCHAIN_MAINTENANCE_1_FEATURES_EXT,
                pNext: core::ptr::null_mut(),
                swapchainMaintenance1: VK_TRUE,
            };

        let device_create_info = VkDeviceCreateInfo {
            sType: VK_STRUCTURE_TYPE_DEVICE_CREATE_INFO,
            pNext: &mut swapchain_maintenance1_features as *mut _ as *mut core::ffi::c_void,
            flags: 0x0,
            queueCreateInfoCount: queue_create_info_count,
            pQueueCreateInfos: queue_create_infos.as_ptr(),
            enabledLayerCount: 0,
            ppEnabledLayerNames: core::ptr::null(),
            enabledExtensionCount: device_extensions.len() as u32,
            ppEnabledExtensionNames: device_extensions.as_ptr(),
            pEnabledFeatures: &physical_device_features,
        };

        println!("Creating device.");
        let mut device = core::ptr::null_mut();
        let result = unsafe {
            vkCreateDevice(
                physical_device,
                &device_create_info,
                core::ptr::null_mut(),
                &mut device,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create vulkan device. Error: {:?}.", result);
        }

        device
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe {
            vkDestroyDevice(self.device, core::ptr::null());
            vkDestroySurfaceKHR(self.instance, self.surface, core::ptr::null());
            vkDestroyInstance(self.instance, core::ptr::null());
        }
    }
}
