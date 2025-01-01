use std::alloc::{self, Layout};
use std::ptr::NonNull;
use crate::error::{Result, SortError};

/// A simple arena allocator for merge sort operations.
/// This reduces allocation overhead by reusing memory.
pub(crate) struct SortArena<T> {
    buffer: NonNull<T>,
    capacity: usize,
    layout: Layout,
}

impl<T> SortArena<T> {
    /// Creates a new arena with the given capacity.
    pub fn new(capacity: usize) -> Result<Self> {
        // Ensure we don't overflow
        let size = std::mem::size_of::<T>()
            .checked_mul(capacity)
            .ok_or_else(|| SortError::allocation_failed(
                "Buffer size overflow",
                None
            ))?;

        // Create layout for allocation
        let layout = Layout::array::<T>(capacity)
            .map_err(|e| SortError::allocation_failed(
                format!("Invalid layout: {}", e),
                None
            ))?;

        // Allocate memory
        let buffer = unsafe {
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                return Err(SortError::allocation_failed(
                    "Failed to allocate buffer",
                    None
                ));
            }
            NonNull::new_unchecked(ptr as *mut T)
        };

        Ok(Self {
            buffer,
            capacity,
            layout,
        })
    }

    /// Gets a mutable slice of the arena's buffer.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.buffer.as_ptr(), self.capacity)
        }
    }
}

impl<T> Drop for SortArena<T> {
    fn drop(&mut self) {
        unsafe {
            alloc::dealloc(self.buffer.as_ptr() as *mut u8, self.layout);
        }
    }
}

/// A buffer for merge operations that handles allocation safely.
pub(crate) struct MergeBuffer<T> {
    data: Vec<T>,
}

impl<T: Clone> MergeBuffer<T> {
    /// Creates a new merge buffer with the given capacity and template value.
    pub fn new(capacity: usize, template: &T) -> Result<Self> {
        let mut data = Vec::new();
        data.try_reserve_exact(capacity)
            .map_err(|e| SortError::allocation_failed(
                format!("Failed to allocate merge buffer of size {}", capacity),
                Some(e)
            ))?;

        // Initialize buffer with clones of the template value
        data.extend(std::iter::repeat_with(|| template.clone()).take(capacity));

        Ok(Self { data })
    }

    /// Gets a mutable slice of the buffer.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    /// Gets a slice of the buffer.
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_buffer_allocation() {
        let buffer = MergeBuffer::new(1000, &42i32).unwrap();
        assert_eq!(buffer.as_slice().len(), 1000);
        assert!(buffer.as_slice().iter().all(|&x| x == 42));
    }

    #[test]
    fn test_merge_buffer_zero_capacity() {
        let buffer = MergeBuffer::<i32>::new(0, &42).unwrap();
        assert_eq!(buffer.as_slice().len(), 0);
    }

    #[test]
    fn test_sort_arena_allocation() {
        let mut arena = SortArena::<i32>::new(1000).unwrap();
        let slice = arena.as_mut_slice();
        assert_eq!(slice.len(), 1000);
    }

    #[test]
    fn test_sort_arena_huge_allocation() {
        // Try to allocate more memory than reasonable
        let result = SortArena::<i32>::new(usize::MAX / 4);
        assert!(result.is_err());
        match result {
            Err(SortError::AllocationFailed { reason, .. }) => {
                assert!(reason.contains("Buffer size overflow") || 
                       reason.contains("Failed to allocate buffer"));
            }
            _ => panic!("Expected allocation failure"),
        }
    }
}