use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
use vk_bindings::*;

/// A Vulkan surface for presenting rendered images to a window.
///
/// Manages a platform-specific window surface and automatically destroys it when dropped.
/// The surface is required for creating swapchains and presenting to the screen.
pub struct VulkanSurface {
    pub(crate) handle: VkSurfaceKHR,
    instance: VkInstance,
}

impl VulkanSurface {
    /// Creates a platform-specific Vulkan surface from a window.
    ///
    /// # Arguments
    /// * `instance` - The Vulkan instance.
    /// * `window` - A window handle that implements HasWindowHandle and HasDisplayHandle.
    pub fn new<H: HasWindowHandle + HasDisplayHandle>(
        instance: VkInstance,
        window: &H,
    ) -> Result<Self, String> {
        let mut handle = std::ptr::null_mut();

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
                    flags: 0x0,
                    dpy: display
                        .display
                        .map(|ptr| ptr.as_ptr())
                        .unwrap_or(core::ptr::null_mut()) as *mut _,
                    window: window.window,
                };
                unsafe {
                    vkCreateXlibSurfaceKHR(instance, &create_info, core::ptr::null(), &mut handle)
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
                    flags: 0x0,
                    connection: display
                        .connection
                        .map(|ptr| ptr.as_ptr())
                        .unwrap_or(core::ptr::null_mut()) as *mut _,
                    window: window.window.get(),
                };
                unsafe {
                    vkCreateXcbSurfaceKHR(instance, &create_info, core::ptr::null(), &mut handle)
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
                    flags: 0x0,
                    display: display.display.as_ptr() as *mut _,
                    surface: window.surface.as_ptr() as *mut _,
                };
                unsafe {
                    vkCreateWaylandSurfaceKHR(
                        instance,
                        &create_info,
                        core::ptr::null(),
                        &mut handle,
                    )
                }
            }
            #[cfg(target_os = "windows")]
            (RawDisplayHandle::Windows(_), RawWindowHandle::Win32(window)) => {
                let create_info = VkWin32SurfaceCreateInfoKHR {
                    sType: VK_STRUCTURE_TYPE_WIN32_SURFACE_CREATE_INFO_KHR,
                    pNext: core::ptr::null(),
                    flags: 0x0,
                    hinstance: window
                        .hinstance
                        .map(|h| h.get() as *mut core::ffi::c_void)
                        .unwrap_or(core::ptr::null_mut()),
                    hwnd: window.hwnd.get() as *mut core::ffi::c_void,
                };
                unsafe {
                    vkCreateWin32SurfaceKHR(instance, &create_info, core::ptr::null(), &mut handle)
                }
            }
            _ => return Err(format!("Unsupported display handle")),
        };

        if result == VK_SUCCESS {
            Ok(Self { handle, instance })
        } else {
            Err(format!("Failed to create surface: {}", result))
        }
    }

    /// Returns the raw Vulkan surface handle.
    pub fn handle(&self) -> VkSurfaceKHR {
        self.handle
    }
}

/// Returns the required Vulkan instance extensions for the current platform.
///
/// Detects the platform from the window's display handle and returns the appropriate
/// surface extension along with the base VK_KHR_surface extension.
///
/// # Arguments
/// * `window` - A window handle implementing HasDisplayHandle.
///
/// # Returns
/// A vector of C string pointers to the required extension names.
pub fn get_required_extensions(window: &impl HasDisplayHandle) -> Vec<*const i8> {
    let display_handle = window.display_handle().unwrap().as_raw();

    let mut extensions = vec![VK_KHR_SURFACE_EXTENSION_NAME.as_ptr() as *const i8];

    match display_handle {
        #[cfg(target_os = "windows")]
        RawDisplayHandle::Windows(_) => {
            extensions.push(VK_KHR_WIN32_SURFACE_EXTENSION_NAME.as_ptr() as *const i8);
        }
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        RawDisplayHandle::Xlib(_) => {
            extensions.push(VK_KHR_XLIB_SURFACE_EXTENSION_NAME.as_ptr() as *const i8);
        }
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        RawDisplayHandle::Xcb(_) => {
            extensions.push(VK_KHR_XCB_SURFACE_EXTENSION_NAME.as_ptr() as *const i8);
        }
        #[cfg(any(target_os = "linux", target_os = "freebsd"))]
        RawDisplayHandle::Wayland(_) => {
            extensions.push(VK_KHR_WAYLAND_SURFACE_EXTENSION_NAME.as_ptr() as *const i8);
        }
        #[cfg(target_os = "macos")]
        RawDisplayHandle::AppKit(_) => {
            extensions.push(VK_EXT_METAL_SURFACE_EXTENSION_NAME.as_ptr() as *const i8);
        }
        _ => panic!("Unsupported platform for Vulkan surface extension"),
    }

    extensions
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        unsafe {
            vkDestroySurfaceKHR(self.instance, self.handle, core::ptr::null());
        }
    }
}
