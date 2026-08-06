use vk_bindings::*;

use crate::backend::vulkan::utils;

/// Alignment utility function
#[inline]
fn align_up(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) & !(alignment - 1)
}

const DEVICE_LOCAL_BLOCK_SIZE: u64 = 256 * 1024 * 1024;
const HOST_VISIBLE_BLOCK_SIZE: u64 = 64 * 1024 * 1024;
const DEDICATED_ALLOCATION_THRESHOLD: u64 = 128 * 1024 * 1024;

/// Represents an allocation from the allocator.
///
/// This handle must be kept alive and passed back to the allocator when freeing.
#[derive(Debug, Clone)]
pub struct Allocation {
    pub(crate) memory: VkDeviceMemory,
    pub(crate) block_index: Option<usize>,
    pub(crate) memory_type_index: u32,
    pub offset: u64,
    pub size: u64,
}

impl Allocation {
    /// Returns true if this is a dedicated allocation (not suballocated)
    pub fn is_dedicated(&self) -> bool {
        self.block_index.is_none()
    }
}

/// Represents a free region within a memory block
#[derive(Debug, Clone)]
struct FreeRegion {
    offset: u64,
    size: u64,
}

/// Represents a single VkDeviceMemory block with suballocations
struct MemoryBlock {
    memory: VkDeviceMemory,
    memory_type_index: u32,
    size: u64,
    used: u64,
    free_regions: Vec<FreeRegion>,
    properties: VkMemoryPropertyFlags,
    mapped_ptr: Option<*mut core::ffi::c_void>,
}

impl MemoryBlock {
    /// Creates a new memory block
    fn new(
        device: VkDevice,
        physical_device: VkPhysicalDevice,
        size: u64,
        memory_type_index: u32,
    ) -> Result<Self, String> {
        let allocate_info = VkMemoryAllocateInfo {
            sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            allocationSize: size,
            memoryTypeIndex: memory_type_index,
        };

        let mut memory = core::ptr::null_mut();
        unsafe {
            let result = vkAllocateMemory(device, &allocate_info, core::ptr::null(), &mut memory);
            if result != VK_SUCCESS {
                return Err(format!("Failed to allocate memory block: {}", result));
            }
        }

        let mut memory_properties = VkPhysicalDeviceMemoryProperties::default();
        unsafe {
            vkGetPhysicalDeviceMemoryProperties(physical_device, &mut memory_properties);
        }
        let properties = memory_properties.memoryTypes[memory_type_index as usize].propertyFlags;

        let mapped_ptr = if (properties & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) != 0 {
            let mut ptr = core::ptr::null_mut();
            unsafe {
                let result = vkMapMemory(device, memory, 0, size, 0, &mut ptr);
                if result != VK_SUCCESS {
                    vkFreeMemory(device, memory, core::ptr::null());
                    return Err(format!(
                        "Failed to persistently map host-visible block: {}",
                        result
                    ));
                }
            }

            Some(ptr)
        } else {
            None
        };

        Ok(Self {
            memory,
            memory_type_index,
            size,
            used: 0,
            free_regions: vec![FreeRegion { offset: 0, size }],
            properties,
            mapped_ptr,
        })
    }

    /// Attempts to allocate from this block with the given size and alignment
    fn allocate(&mut self, size: u64, alignment: u64) -> Option<Allocation> {
        for i in 0..self.free_regions.len() {
            let region_offset = self.free_regions[i].offset;
            let region_size = self.free_regions[i].size;
            let aligned_offset = align_up(region_offset, alignment);
            let padding = aligned_offset - region_offset;

            if padding + size <= region_size {
                let allocation = Allocation {
                    memory: self.memory,
                    offset: aligned_offset,
                    size,
                    block_index: None, // Set by VulkanAllocator after insertion
                    memory_type_index: self.memory_type_index,
                };

                let remaining_size = region_size - padding - size;
                let remaining_offset = aligned_offset + size;

                self.free_regions.remove(i);

                if padding > 0 {
                    self.free_regions.push(FreeRegion {
                        offset: region_offset,
                        size: padding,
                    });
                }

                if remaining_size > 0 {
                    self.free_regions.push(FreeRegion {
                        offset: remaining_offset,
                        size: remaining_size,
                    });
                }

                self.free_regions.sort_by_key(|r| r.offset);

                self.used += size;
                return Some(allocation);
            }
        }

        None
    }

    /// Frees an allocation within this block
    fn free(&mut self, allocation: &Allocation) {
        if allocation.memory != self.memory {
            return; // Not in the same block
        }

        self.free_regions.push(FreeRegion {
            offset: allocation.offset,
            size: allocation.size,
        });

        self.free_regions.sort_by_key(|r| r.offset);

        let mut i = 0;
        while i < self.free_regions.len().saturating_sub(1) {
            let current_end = self.free_regions[i].offset + self.free_regions[i].size;
            let next_start = self.free_regions[i + 1].offset;

            if current_end == next_start {
                self.free_regions[i].size += self.free_regions[i + 1].size;
                self.free_regions.remove(i + 1);
            } else {
                i += 1;
            }
        }

        self.used = self.used.saturating_sub(allocation.size);
    }

    /// Returns true if the entire block is free
    fn is_empty(&self) -> bool {
        self.used == 0
    }

    /// Returns the base mapped pointer for host-visible blocks, or None for device-local.
    /// The caller must add the allocation's byte offset to reach the correct address.
    fn mapped_base(&self) -> Option<*mut core::ffi::c_void> {
        self.mapped_ptr
    }

    /// Unmaps the block memory
    fn unmap(&self, device: VkDevice) {
        if self.mapped_ptr.is_some() {
            unsafe {
                vkUnmapMemory(device, self.memory);
            }
        }
    }
}

/// Custom Vulkan memory allocator using block allocation strategy.
///
/// Manages Vulkan memory efficiently by allocating large blocks and suballocating
/// from them, reducing the number of vkAllocateMemory calls and avoiding the
/// per-device allocation limit.
pub struct VulkanAllocator {
    device: VkDevice,
    physical_device: VkPhysicalDevice,
    blocks: Vec<Option<MemoryBlock>>,
}

impl VulkanAllocator {
    /// Creates a new Vulkan allocator
    ///
    /// # Arguments
    /// * `device` - The Vulkan logical device
    /// * `physical_device` - The Vulkan physical device
    pub fn new(device: VkDevice, physical_device: VkPhysicalDevice) -> Self {
        Self {
            device,
            physical_device,
            blocks: Vec::new(),
        }
    }

    /// Allocates memory with the given requirements
    ///
    /// # Arguments
    /// * `size` - Size in bytes
    /// * `alignment` - Required alignment
    /// * `memory_type_bits` - Memory type bits from VkMemoryRequirements
    /// * `properties` - Desired memory property flags
    ///
    /// # Returns
    /// An Allocation handle or an error
    pub fn allocate(
        &mut self,
        size: u64,
        alignment: u64,
        memory_type_bits: u32,
        properties: VkMemoryPropertyFlags,
    ) -> Result<Allocation, String> {
        let memory_type_index =
            utils::find_memory_type(self.physical_device, memory_type_bits, properties)?;

        if size >= DEDICATED_ALLOCATION_THRESHOLD {
            return self.allocate_dedicated(size, memory_type_index);
        }

        // Try to sub-allocate from an existing block of the matching memory type.
        for (slot_index, slot) in self.blocks.iter_mut().enumerate() {
            if let Some(block) = slot {
                if block.memory_type_index == memory_type_index {
                    if let Some(mut allocation) = block.allocate(size, alignment) {
                        allocation.block_index = Some(slot_index);

                        return Ok(allocation);
                    }
                }
            }
        }

        // No suitable block found, create a new one
        let block_size = self.get_block_size(properties);
        let mut new_block = MemoryBlock::new(
            self.device,
            self.physical_device,
            block_size,
            memory_type_index,
        )?;

        let mut allocation = new_block
            .allocate(size, alignment)
            .ok_or_else(|| "Failed to allocate from new block".to_string())?;

        // Fill a reclaimed None slot if one exists, otherwise grow the Vec.
        let slot_index = if let Some(free_slot) = self.blocks.iter().position(|s| s.is_none()) {
            self.blocks[free_slot] = Some(new_block);
            free_slot
        } else {
            let index = self.blocks.len();
            self.blocks.push(Some(new_block));
            index
        };

        allocation.block_index = Some(slot_index);

        Ok(allocation)
    }

    /// Frees a previously allocated allocation
    ///
    /// # Arguments
    /// * `allocation` - The allocation to free
    pub fn free(&mut self, allocation: &Allocation) {
        if let Some(block_index) = allocation.block_index {
            if let Some(Some(block)) = self.blocks.get_mut(block_index) {
                block.free(allocation);

                if block.is_empty() {
                    let block = self.blocks[block_index].take().unwrap();
                    block.unmap(self.device);
                    unsafe {
                        vkFreeMemory(self.device, block.memory, core::ptr::null());
                    }
                }
            }
        } else {
            // Dedicated allocation
            unsafe {
                vkFreeMemory(self.device, allocation.memory, core::ptr::null());
            }
        }
    }

    /// Maps memory for CPU access
    ///
    /// # Arguments
    /// * `allocation` - The allocation to map
    ///
    /// # Returns
    /// Pointer to mapped memory (already offset to allocation start)
    pub fn map(&self, allocation: &Allocation) -> Result<*mut core::ffi::c_void, String> {
        if let Some(block_index) = allocation.block_index {
            match self.blocks.get(block_index) {
                Some(Some(block)) => match block.mapped_base() {
                    Some(base_ptr) => Ok(unsafe {
                        (base_ptr as *mut u8).add(allocation.offset as usize) as *mut _
                    }),
                    None => {
                        Err("Attempted to map a non-host-visible suballocated buffer".to_string())
                    }
                },
                _ => Err(format!("Invalid block index {}", block_index)),
            }
        } else {
            // Dedicated allocation
            let mut data = core::ptr::null_mut();
            unsafe {
                let result = vkMapMemory(
                    self.device,
                    allocation.memory,
                    allocation.offset,
                    allocation.size,
                    0,
                    &mut data,
                );
                if result != VK_SUCCESS {
                    return Err(format!("Failed to map memory: {}", result));
                }
            }

            Ok(data)
        }
    }

    /// Unmaps previously mapped memory
    ///
    /// # Arguments
    /// * `allocation` - The allocation to unmap
    pub fn unmap(&self, allocation: &Allocation) {
        if allocation.block_index.is_none() {
            unsafe {
                vkUnmapMemory(self.device, allocation.memory);
            }
        }
    }

    /// Gets the appropriate block size for the memory properties
    fn get_block_size(&self, properties: VkMemoryPropertyFlags) -> u64 {
        if (properties & VK_MEMORY_PROPERTY_HOST_VISIBLE_BIT) != 0 {
            HOST_VISIBLE_BLOCK_SIZE
        } else {
            DEVICE_LOCAL_BLOCK_SIZE
        }
    }

    /// Allocates dedicated memory (not suballocated)
    fn allocate_dedicated(
        &mut self,
        size: u64,
        memory_type_index: u32,
    ) -> Result<Allocation, String> {
        let allocate_info = VkMemoryAllocateInfo {
            sType: VK_STRUCTURE_TYPE_MEMORY_ALLOCATE_INFO,
            pNext: core::ptr::null(),
            allocationSize: size,
            memoryTypeIndex: memory_type_index,
        };

        let mut memory = core::ptr::null_mut();
        unsafe {
            let result =
                vkAllocateMemory(self.device, &allocate_info, core::ptr::null(), &mut memory);
            if result != VK_SUCCESS {
                return Err(format!("Failed to allocate dedicated memory: {}", result));
            }
        }

        Ok(Allocation {
            memory,
            offset: 0,
            size,
            block_index: None,
            memory_type_index,
        })
    }
}

impl Drop for VulkanAllocator {
    fn drop(&mut self) {
        unsafe {
            for slot in &self.blocks {
                if let Some(block) = slot {
                    block.unmap(self.device);
                    vkFreeMemory(self.device, block.memory, core::ptr::null());
                }
            }
        }
    }
}
