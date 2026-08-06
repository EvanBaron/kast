use crate::backend::vulkan::allocator::{Allocation, VulkanAllocator};

/// Deletion queue for deferred resource cleanup.
///
/// Resources in Vulkan cannot be destroyed immediately when they go out of scope
/// because they might still be in use by the GPU. The deletion queue defers
/// cleanup until it's safe to destroy resources.
///
/// With N frames in flight, a resource marked for deletion in frame K will be
/// actually destroyed when frame K completes.
pub struct DeletionQueue {
    queues: Vec<Vec<Box<dyn FnOnce()>>>,
    allocations: Vec<Vec<Allocation>>,
    current_frame: usize,
    frames_in_flight: usize,
}

impl DeletionQueue {
    /// Creates a new deletion queue with the specified number of frames in flight.
    ///
    /// # Arguments
    /// * `frames_in_flight` - Number of frames that can be in flight simultaneously (typically 2-3)
    ///
    /// # Panics
    /// Panics if frames_in_flight is 0
    pub fn new(frames_in_flight: usize) -> Self {
        assert!(
            frames_in_flight > 0,
            "frames_in_flight must be greater than 0"
        );

        let mut queues = Vec::with_capacity(frames_in_flight);
        let mut allocations = Vec::with_capacity(frames_in_flight);
        for _ in 0..frames_in_flight {
            queues.push(Vec::new());
            allocations.push(Vec::new());
        }

        Self {
            queues,
            allocations,
            current_frame: 0,
            frames_in_flight,
        }
    }

    /// Pushes a deletion function to the current frame's queue.
    ///
    /// The function will be executed when the current frame completes
    ///
    /// # Arguments
    /// * `deleter` - A closure that performs the cleanup
    pub fn push<F>(&mut self, deleter: F)
    where
        F: FnOnce() + 'static,
    {
        self.queues[self.current_frame].push(Box::new(deleter));
    }

    /// Pushes a deletion function along with an allocation to free.
    ///
    /// The function will be executed when the current frame completes,
    /// and the allocation will be freed afterwards.
    ///
    /// # Arguments
    /// * `allocation` - The allocation to free after the deleter runs
    /// * `deleter` - A closure that performs the cleanup
    pub fn push_with_allocation<F>(&mut self, allocation: Allocation, deleter: F)
    where
        F: FnOnce() + 'static,
    {
        self.queues[self.current_frame].push(Box::new(deleter));
        self.allocations[self.current_frame].push(allocation);
    }

    /// Advances to the next frame and flushes the new frame's deletion queue.
    ///
    /// This should be called at the beginning of each frame, after waiting for
    /// the frame's fence to signal (ensuring the GPU is done with that frame).
    ///
    /// All pending deletions for this frame will be executed and allocations freed.
    ///
    /// # Arguments
    /// * `allocator` - The allocator to free memory with
    pub fn next_frame(&mut self, allocator: &mut VulkanAllocator) {
        self.current_frame = (self.current_frame + 1) % self.frames_in_flight;
        self.flush_current(allocator);
    }

    /// Flushes the current frame's deletion queue without advancing.
    ///
    /// Executes and clears all pending deletions for the current frame,
    /// then frees all allocations.
    ///
    /// # Arguments
    /// * `allocator` - The allocator to free memory with
    pub fn flush_current(&mut self, allocator: &mut VulkanAllocator) {
        let queue = &mut self.queues[self.current_frame];
        for deleter in queue.drain(..) {
            deleter();
        }

        let allocations = &mut self.allocations[self.current_frame];
        for allocation in allocations.drain(..) {
            allocator.free(&allocation);
        }
    }

    /// Flushes all deletion queues for all frames.
    ///
    /// This should be called during shutdown after ensuring the device is idle.
    /// Executes all pending deletions across all frames and frees all allocations.
    ///
    /// # Arguments
    /// * `allocator` - The allocator to free memory with
    pub fn flush_all(&mut self, allocator: &mut VulkanAllocator) {
        for queue in &mut self.queues {
            for deleter in queue.drain(..) {
                deleter();
            }
        }

        for allocations in &mut self.allocations {
            for allocation in allocations.drain(..) {
                allocator.free(&allocation);
            }
        }
    }

    /// Returns the current frame index
    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    /// Returns the number of frames in flight
    pub fn frames_in_flight(&self) -> usize {
        self.frames_in_flight
    }

    /// Returns the number of pending deletions for the current frame
    pub fn pending_deletions(&self) -> usize {
        self.queues[self.current_frame].len()
    }

    /// Returns the total number of pending deletions across all frames
    pub fn total_pending_deletions(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum()
    }

    /// Returns the total number of pending allocations to free across all frames
    pub fn total_pending_allocations(&self) -> usize {
        self.allocations.iter().map(|a| a.len()).sum()
    }
}

impl Drop for DeletionQueue {
    fn drop(&mut self) {
        if self.total_pending_deletions() > 0 || self.total_pending_allocations() > 0 {
            panic!(
                "DeletionQueue dropped with pending work! Call flush_all() before dropping. \
                 Pending deletions: {}, Pending allocations: {}",
                self.total_pending_deletions(),
                self.total_pending_allocations()
            );
        }
    }
}
