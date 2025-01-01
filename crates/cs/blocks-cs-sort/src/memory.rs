use std::alloc::{self, Layout};
use std::ptr::{NonNull, addr_of_mut};
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use crate::error::{Result, SortError};

/// A simple arena allocator for merge sort operations.
/// This reduces allocation overhead by reusing memory.
/// 
/// # Safety
/// 
/// This type uses raw pointers internally and requires several safety invariants:
/// - The buffer must be properly aligned for type T
/// - The buffer must be properly initialized before use
/// - The buffer must not exceed isize::MAX bytes
/// - The type T must be properly dropped when the arena is dropped
/// 
/// These invariants are maintained by the public API and checked at runtime
/// where possible.
pub(crate) struct SortArena<T> {
    /// Raw pointer to the allocated memory
    buffer: NonNull<T>,
    /// Number of elements the buffer can hold
    capacity: usize,
    /// Layout used for allocation/deallocation
    layout: Layout,
    /// Marker for the generic type
    _marker: PhantomData<T>,
}

/// Compute the layout for an array of T with proper alignment
fn array_layout<T>(capacity: usize) -> Result<Layout> {
    // Check for zero capacity
    if capacity == 0 {
        return Err(SortError::allocation_failed(
            "Cannot allocate arena with zero capacity",
            None
        ));
    }

    // Check total size against isize::MAX
    let element_size = mem::size_of::<T>();
    let total_size = element_size
        .checked_mul(capacity)
        .ok_or_else(|| SortError::allocation_failed(
            "Buffer size overflow",
            None
        ))?;

    if total_size > isize::MAX as usize {
        return Err(SortError::allocation_failed(
            format!("Total size {} exceeds isize::MAX", total_size),
            None
        ));
    }

    // Create layout with proper alignment
    Layout::array::<T>(capacity)
        .map_err(|e| SortError::allocation_failed(
            format!("Invalid layout: {}", e),
            None
        ))
}

impl<T> SortArena<T> {
    /// Creates a new arena with the given capacity.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Capacity is zero
    /// - Total size would exceed isize::MAX
    /// - Memory allocation fails
    /// - Layout is invalid for type T
    pub fn new(capacity: usize) -> Result<Self> {
        // Get layout with proper checks
        let layout = array_layout::<T>(capacity)?;

        // Allocate memory
        let buffer = unsafe {
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                alloc::handle_alloc_error(layout);
            }
            // SAFETY: ptr is non-null as checked above
            NonNull::new_unchecked(ptr as *mut T)
        };

        Ok(Self {
            buffer,
            capacity,
            layout,
            _marker: PhantomData,
        })
    }

    /// Gets a mutable slice of the arena's buffer.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that:
    /// - The buffer is properly initialized
    /// - No other references to the buffer exist
    pub unsafe fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: buffer is valid for capacity elements as guaranteed by new()
        std::slice::from_raw_parts_mut(self.buffer.as_ptr(), self.capacity)
    }

    /// Gets a slice of the arena's buffer.
    /// 
    /// # Safety
    /// 
    /// The caller must ensure that:
    /// - The buffer is properly initialized
    pub unsafe fn as_slice(&self) -> &[T] {
        // SAFETY: buffer is valid for capacity elements as guaranteed by new()
        std::slice::from_raw_parts(self.buffer.as_ptr(), self.capacity)
    }

    /// Returns the capacity of the arena.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns true if the arena is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.capacity == 0
    }

    /// Returns the memory layout used by the arena.
    #[inline]
    pub fn layout(&self) -> Layout {
        self.layout
    }
}

impl<T> Drop for SortArena<T> {
    fn drop(&mut self) {
        // SAFETY: buffer was allocated with the same layout in new()
        unsafe {
            alloc::dealloc(self.buffer.as_ptr() as *mut u8, self.layout);
        }
    }
}

// SAFETY: T determines thread safety. If T is Send, the raw pointer can be sent
// between threads safely because we maintain the invariant that it always points
// to a valid allocation of T.
unsafe impl<T: Send> Send for SortArena<T> {}

// SAFETY: T determines thread safety. If T is Sync, the raw pointer can be shared
// between threads safely because we maintain the invariant that it always points
// to a valid allocation of T and all mutations require exclusive access.
unsafe impl<T: Sync> Sync for SortArena<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_arena_allocation() {
        let mut arena = SortArena::<i32>::new(1000).unwrap();
        assert_eq!(arena.capacity(), 1000);
        assert!(!arena.is_empty());
    }

    #[test]
    fn test_arena_zero_capacity() {
        let result = SortArena::<i32>::new(0);
        assert!(result.is_err());
        match result {
            Err(SortError::AllocationFailed { reason, .. }) => {
                assert!(reason.contains("zero capacity"));
            }
            _ => panic!("Expected allocation failure"),
        }
    }

    #[test]
    fn test_arena_huge_allocation() {
        let result = SortArena::<i32>::new(usize::MAX / 4);
        assert!(result.is_err());
        match result {
            Err(SortError::AllocationFailed { reason, .. }) => {
                assert!(reason.contains("overflow") || 
                       reason.contains("exceeds isize::MAX"));
            }
            _ => panic!("Expected allocation failure"),
        }
    }

    #[test]
    fn test_arena_thread_safety() {
        // Test Send
        let arena = SortArena::<i32>::new(100).unwrap();
        thread::spawn(move || {
            assert_eq!(arena.capacity(), 100);
        }).join().unwrap();

        // Test Sync
        let arena = Arc::new(SortArena::<i32>::new(100).unwrap());
        let arena2 = arena.clone();
        thread::spawn(move || {
            assert_eq!(arena2.capacity(), 100);
        }).join().unwrap();
    }

    #[test]
    fn test_arena_layout() {
        let arena = SortArena::<i32>::new(100).unwrap();
        let layout = arena.layout();
        assert_eq!(layout.size(), 100 * std::mem::size_of::<i32>());
        assert_eq!(layout.align(), std::mem::align_of::<i32>());
    }

    // Static assertions for thread safety
    static_assertions::assert_impl_all!(SortArena<i32>: Send, Sync);
    static_assertions::assert_not_impl_any!(SortArena<*const i32>: Send, Sync);
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