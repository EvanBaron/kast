use crate::graphics::utils;
use vk_bindings::*;

pub const IMAGE_FORMAT: VkFormat = VK_FORMAT_R8G8B8A8_SRGB;
pub const BYTES_PER_PIXEL: usize = 4 * core::mem::size_of::<u8>();

const IMAGE_MEM_PROPS: VkMemoryPropertyFlags =
    VK_MEMORY_PROPERTY_DEVICE_LOCAL_BIT as VkMemoryPropertyFlags;

pub struct ImageData {
    pub bytes_per_pixel: usize,
    pub width: usize,
    pub height: usize,
    pub data: Vec<u8>,
}

impl ImageData {
    pub fn get_data_size(&self) -> usize {
        self.width * self.height * self.bytes_per_pixel
    }

    pub fn get_pixel(&mut self, x: usize, y: usize) -> &mut [u8] {
        let begin = (y * self.width + x) * self.bytes_per_pixel;
        let end = (y * self.width + x + 1) * self.bytes_per_pixel;

        &mut self.data[begin..end]
    }
}

pub struct Image {
    pub handle: VkImage,
    pub image_view: VkImageView,
    pub memory: VkDeviceMemory,
    pub width: u32,
    pub height: u32,
    device: VkDevice,
}

impl Image {
    pub fn new(
        device: VkDevice,
        physical_device: VkPhysicalDevice,
        image_data: &ImageData,
    ) -> Self {
        let image_create_info = VkImageCreateInfo {
            sType: VK_STRUCTURE_TYPE_IMAGE_CREATE_INFO,
            pNext: core::ptr::null(),
            flags: 0x0,
            imageType: VK_IMAGE_TYPE_2D,
            format: IMAGE_FORMAT,
            extent: VkExtent3D {
                width: image_data.width as u32,
                height: image_data.height as u32,
                depth: 1,
            },
            mipLevels: 1,
            arrayLayers: 1,
            samples: VK_SAMPLE_COUNT_1_BIT,
            tiling: VK_IMAGE_TILING_OPTIMAL,
            usage: (VK_IMAGE_USAGE_SAMPLED_BIT | VK_IMAGE_USAGE_TRANSFER_DST_BIT)
                as VkImageUsageFlags,
            sharingMode: VK_SHARING_MODE_EXCLUSIVE,
            queueFamilyIndexCount: 0,
            pQueueFamilyIndices: core::ptr::null(),
            initialLayout: VK_IMAGE_LAYOUT_UNDEFINED,
        };

        println!("Creating image.");
        let mut handle = core::ptr::null_mut();
        let result = unsafe {
            vkCreateImage(
                device,
                &image_create_info,
                core::ptr::null_mut(),
                &mut handle,
            )
        };

        if result != VK_SUCCESS {
            panic!("Failed to create image. Error: {}", result);
        }

        let mut memory_requirements = VkMemoryRequirements::default();
        unsafe {
            vkGetImageMemoryRequirements(device, handle, &mut memory_requirements);
        }

        let memory_type = utils::find_memory_type(
            physical_device,
            memory_requirements.memoryTypeBits,
            IMAGE_MEM_PROPS,
        );

        let image_allocate_info = VkMemoryAllocateInfo {
            sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            allocationSize: memory_requirements.size,
            memoryTypeIndex: memory_type,
        };

        let mut memory = core::ptr::null_mut();
        let result = unsafe {
            vkAllocateMemory(device, &image_allocate_info, core::ptr::null(), &mut memory)
        };

        if result != VK_SUCCESS {
            panic!("Could not allocate memory. Error: {}", result);
        }

        let result = unsafe { vkBindImageMemory(device, handle, memory, 0) };

        if result != VK_SUCCESS {
            panic!("Failed to bind memory to image. Error: {}", result);
        }

        println!("Creating image view.");
        let image_view = utils::create_image_view(
            device,
            handle,
            IMAGE_FORMAT,
            VK_IMAGE_ASPECT_COLOR_BIT as u32,
        );

        Self {
            handle,
            image_view,
            memory,
            width: image_data.width as u32,
            height: image_data.height as u32,
            device,
        }
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        unsafe {
            println!("Deleting image");
            vkDestroyImageView(self.device, self.image_view, core::ptr::null_mut());
            vkDestroyImage(self.device, self.handle, core::ptr::null_mut());
            vkFreeMemory(self.device, self.memory, core::ptr::null_mut());
        }
    }
}
