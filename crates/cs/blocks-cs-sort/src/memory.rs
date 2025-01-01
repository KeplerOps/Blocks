use std::alloc::{self, Layout};
use std::ptr::{NonNull, addr_of_mut};
use std::marker::PhantomData;
use std::mem::{self, MaybeUninit};
use crate::error::{Result, SortError};

/// Cache line size for the current architecture.
#[cfg(target_arch = "x86_64")]
const CACHE_LINE_SIZE: usize = 64;
#[cfg(target_arch = "aarch64")]
const CACHE_LINE_SIZE: usize = 64;
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const CACHE_LINE_SIZE: usize = 32;

/// SIMD vector size for the current architecture.
#[cfg(target_arch = "x86_64")]
const SIMD_WIDTH: usize = 32; // AVX-256
#[cfg(target_arch = "aarch64")]
const SIMD_WIDTH: usize = 16; // NEON
#[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
const SIMD_WIDTH: usize = 16;

/// A simple arena allocator for merge sort operations.
/// This reduces allocation overhead by reusing memory.
/// 
/// # Safety
/// 
/// This type uses raw pointers internally and requires several safety invariants:
/// - The buffer must be properly aligned for type T and SIMD operations
/// - The buffer must be properly initialized before use
/// - The buffer must not exceed isize::MAX bytes
/// - The type T must be properly dropped when the arena is dropped
/// 
/// These invariants are maintained by the public API and checked at runtime
/// where possible.
/// 
/// # Memory Layout
/// 
/// The arena ensures optimal memory layout for performance:
/// - Buffer is aligned to cache line boundaries (64 bytes on x86_64/aarch64)
/// - Buffer size is padded to SIMD vector width when possible
/// - Elements are properly aligned for their type requirements
/// 
/// ```text
/// Memory Layout:
/// ┌────────────────────────────────────────┐
/// │ Cache Line Aligned Buffer              │
/// ├────────────────────┬──────────────────┤
/// │ SIMD Vector 1      │ SIMD Vector 2    │
/// ├────────────────────┼──────────────────┤
/// │ Elements 1-8       │ Elements 9-16    │
/// └────────────────────┴──────────────────┘
/// ```
#[repr(C, align(64))]  // Align to cache line
pub(crate) struct SortArena<T> {
    /// Raw pointer to the allocated memory (SIMD aligned)
    buffer: NonNull<T>,
    /// Number of elements the buffer can hold
    capacity: usize,
    /// Layout used for allocation/deallocation
    layout: Layout,
    /// Marker for the generic type
    _marker: PhantomData<T>,
}

/// Compute the layout for an array of T with proper alignment for SIMD and cache efficiency.
/// 
/// This function ensures:
/// 1. The buffer is aligned to cache line boundaries
/// 2. Each element is properly aligned for its type
/// 3. The total size is padded for SIMD operations
/// 4. Memory limits are respected
fn array_layout<T>(capacity: usize) -> Result<Layout> {
    // Check for zero capacity
    if capacity == 0 {
        return Err(SortError::allocation_failed(
            "Cannot allocate arena with zero capacity",
            None
        ));
    }

    // Get element properties
    let element_size = mem::size_of::<T>();
    let element_align = mem::align_of::<T>();

    // Calculate required alignment (max of cache line, SIMD width, and type alignment)
    let required_align = CACHE_LINE_SIZE
        .max(SIMD_WIDTH)
        .max(element_align);

    // Calculate padded capacity for SIMD operations
    let simd_elements = SIMD_WIDTH / element_size;
    let padded_capacity = if simd_elements > 1 {
        // Round up to nearest SIMD vector size
        (capacity + simd_elements - 1) & !(simd_elements - 1)
    } else {
        capacity
    };

    // Check total size against isize::MAX
    let total_size = element_size
        .checked_mul(padded_capacity)
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
    let layout = Layout::from_size_align(total_size, required_align)
        .map_err(|e| SortError::allocation_failed(
            format!("Invalid layout: {}", e),
            None
        ))?;

    // Verify alignment requirements are met
    debug_assert!(layout.align() >= element_align, 
        "Layout alignment {} is less than type alignment {}", 
        layout.align(), element_align);
    debug_assert!(layout.align() >= SIMD_WIDTH,
        "Layout alignment {} is less than SIMD width {}", 
        layout.align(), SIMD_WIDTH);
    debug_assert!(layout.size() % SIMD_WIDTH == 0,
        "Layout size {} is not a multiple of SIMD width {}", 
        layout.size(), SIMD_WIDTH);

    Ok(layout)
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
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::alloc::Layout;
    use std::mem::{self, MaybeUninit};

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

    mod alignment_tests {
        use super::*;

        #[test]
        fn test_basic_alignment() {
            // Test with a type that requires strict alignment
            #[repr(align(16))]
            #[derive(Debug)]
            struct Aligned([u8; 32]);

            let arena = SortArena::<Aligned>::new(10).unwrap();
            let ptr = arena.buffer.as_ptr();
            
            // Check pointer alignment
            assert_eq!(ptr as usize % 16, 0, "Buffer not properly aligned");
            
            // Check layout alignment
            assert_eq!(arena.layout().align(), 64); // Should be cache line aligned
        }

        #[test]
        fn test_simd_alignment() {
            let arena = SortArena::<i32>::new(100).unwrap();
            let ptr = arena.buffer.as_ptr();
            let layout = arena.layout();

            // Check SIMD alignment requirements
            assert!(layout.align() >= SIMD_WIDTH);
            assert_eq!(ptr as usize % SIMD_WIDTH, 0);
            assert_eq!(layout.size() % SIMD_WIDTH, 0);
        }

        #[test]
        fn test_cache_line_alignment() {
            let arena = SortArena::<u8>::new(1000).unwrap();
            let ptr = arena.buffer.as_ptr();
            let layout = arena.layout();

            // Check cache line alignment
            assert!(layout.align() >= CACHE_LINE_SIZE);
            assert_eq!(ptr as usize % CACHE_LINE_SIZE, 0);
        }

        #[test]
        fn test_padded_capacity() {
            // Test with a type smaller than SIMD width
            let size = 10;
            let arena = SortArena::<i32>::new(size).unwrap();
            let layout = arena.layout();

            // Should be padded to SIMD width
            let simd_elements = SIMD_WIDTH / std::mem::size_of::<i32>();
            let expected_capacity = ((size + simd_elements - 1) / simd_elements) * simd_elements;
            assert_eq!(layout.size(), expected_capacity * std::mem::size_of::<i32>());
        }

        #[test]
        fn test_large_alignment() {
            // Test with a large alignment requirement
            #[repr(align(128))]
            #[derive(Debug)]
            struct LargeAlign([u8; 256]);

            let arena = SortArena::<LargeAlign>::new(5).unwrap();
            let ptr = arena.buffer.as_ptr();
            let layout = arena.layout();

            // Should respect both type alignment and cache line size
            assert!(layout.align() >= 128);
            assert!(layout.align() >= CACHE_LINE_SIZE);
            assert_eq!(ptr as usize % 128, 0);
        }

        #[test]
        fn test_mixed_size_alignment() {
            // Test with types of different sizes
            let arena1 = SortArena::<u8>::new(1000).unwrap();
            let arena2 = SortArena::<u16>::new(500).unwrap();
            let arena3 = SortArena::<u32>::new(250).unwrap();
            let arena4 = SortArena::<u64>::new(125).unwrap();

            // All should be cache line and SIMD aligned
            for arena in [&arena1, &arena2, &arena3, &arena4] {
                let ptr = arena.buffer.as_ptr() as usize;
                let layout = arena.layout();

                assert!(layout.align() >= CACHE_LINE_SIZE);
                assert!(layout.align() >= SIMD_WIDTH);
                assert_eq!(ptr % CACHE_LINE_SIZE, 0);
                assert_eq!(ptr % SIMD_WIDTH, 0);
            }
        }

        #[test]
        fn test_alignment_stress() {
            // Test alignment with various sizes around SIMD boundaries
            let sizes = [
                SIMD_WIDTH - 1,
                SIMD_WIDTH,
                SIMD_WIDTH + 1,
                SIMD_WIDTH * 2 - 1,
                SIMD_WIDTH * 2,
                SIMD_WIDTH * 2 + 1,
            ];

            for &size in &sizes {
                let arena = SortArena::<u8>::new(size).unwrap();
                let layout = arena.layout();

                // Check alignment requirements
                assert!(layout.align() >= CACHE_LINE_SIZE);
                assert!(layout.size() % SIMD_WIDTH == 0);
                assert_eq!(arena.buffer.as_ptr() as usize % SIMD_WIDTH, 0);
            }
        }
    }

    #[test]
    fn test_arena_drop() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
        
        struct DropCounter;
        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Create and drop arena
        {
            let size = 100;
            let arena = SortArena::<DropCounter>::new(size).unwrap();
            unsafe {
                // Initialize counters
                let slice = std::slice::from_raw_parts_mut(
                    arena.buffer.as_ptr(),
                    size
                );
                for i in 0..size {
                    std::ptr::write(&mut slice[i], DropCounter);
                }
            }
            // Arena drops here
        }

        // Verify all items were dropped
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 100);
    }

    #[test]
    fn test_arena_concurrent_access() {
        let size = 1000;
        let arena = Arc::new(SortArena::<i32>::new(size).unwrap());
        let threads = 4;
        let barrier = Arc::new(Barrier::new(threads));
        
        // Spawn threads that read from different parts of the arena
        let handles: Vec<_> = (0..threads)
            .map(|i| {
                let arena = arena.clone();
                let barrier = barrier.clone();
                let chunk_size = size / threads;
                let start = i * chunk_size;
                let end = start + chunk_size;
                
                thread::spawn(move || {
                    barrier.wait();
                    unsafe {
                        let slice = std::slice::from_raw_parts(
                            arena.buffer.as_ptr().add(start),
                            chunk_size
                        );
                        // Just read the memory to check for race conditions
                        let _ = std::ptr::read_volatile(&slice[0]);
                    }
                })
            })
            .collect();

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    // Memory leak test using custom allocator
    #[test]
    fn test_arena_memory_leaks() {
        use std::alloc::System;
        
        struct LeakDetector {
            allocations: AtomicUsize,
            deallocations: AtomicUsize,
        }

        impl LeakDetector {
            const fn new() -> Self {
                Self {
                    allocations: AtomicUsize::new(0),
                    deallocations: AtomicUsize::new(0),
                }
            }
        }

        unsafe impl GlobalAlloc for LeakDetector {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                self.allocations.fetch_add(1, Ordering::SeqCst);
                System.alloc(layout)
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                self.deallocations.fetch_add(1, Ordering::SeqCst);
                System.dealloc(ptr, layout)
            }
        }

        #[global_allocator]
        static LEAK_DETECTOR: LeakDetector = LeakDetector::new();

        // Reset counters
        LEAK_DETECTOR.allocations.store(0, Ordering::SeqCst);
        LEAK_DETECTOR.deallocations.store(0, Ordering::SeqCst);

        // Create and drop arena
        {
            let _arena = SortArena::<i32>::new(100).unwrap();
        }

        // Verify all allocations were freed
        assert_eq!(
            LEAK_DETECTOR.allocations.load(Ordering::SeqCst),
            LEAK_DETECTOR.deallocations.load(Ordering::SeqCst),
            "Memory leak detected"
        );
    }

    // Static assertions for thread safety
    static_assertions::assert_impl_all!(SortArena<i32>: Send, Sync);
    static_assertions::assert_not_impl_any!(SortArena<*const i32>: Send, Sync);
}

/// A buffer for merge operations that handles allocation safely.
/// 
/// This type provides optimized implementations for different types:
/// - Copy types use direct memory copying
/// - Primitive types can use SIMD operations
/// - Other types fall back to clone-based initialization
#[derive(Debug)]
pub(crate) struct MergeBuffer<T> {
    data: Vec<T>,
}

impl<T: Clone> MergeBuffer<T> {
    /// Creates a new merge buffer with the given capacity and template value.
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - Capacity would exceed isize::MAX bytes
    /// - Memory allocation fails
    pub fn new(capacity: usize, template: &T) -> Result<Self> {
        // Check capacity
        if capacity > 0 {
            // Ensure we don't exceed isize::MAX bytes
            let size = std::mem::size_of::<T>()
                .checked_mul(capacity)
                .ok_or_else(|| SortError::allocation_failed(
                    "Buffer size overflow",
                    None
                ))?;

            if size > isize::MAX as usize {
                return Err(SortError::allocation_failed(
                    format!("Total size {} exceeds isize::MAX", size),
                    None
                ));
            }
        }

        // Allocate and initialize
        let mut data = Vec::new();
        data.try_reserve_exact(capacity)
            .map_err(|e| SortError::allocation_failed(
                format!("Failed to allocate merge buffer of size {}", capacity),
                Some(e)
            ))?;

        // Initialize buffer
        if capacity > 0 {
            Self::initialize_buffer(&mut data, capacity, template)?;
        }

        Ok(Self { data })
    }

    /// Gets a mutable slice of the buffer.
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    /// Gets a slice of the buffer.
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self.data.as_slice()
    }

    /// Returns the capacity of the buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Returns true if the buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Initialize the buffer with the template value.
    #[inline]
    fn initialize_buffer(data: &mut Vec<T>, capacity: usize, template: &T) -> Result<()> {
        data.extend(std::iter::repeat_with(|| template.clone()).take(capacity));
        Ok(())
    }
}

// Specialization for Copy types
impl<T: Copy> MergeBuffer<T> {
    /// Initialize the buffer using memcpy for Copy types.
    #[inline]
    fn initialize_buffer(data: &mut Vec<T>, capacity: usize, template: &T) -> Result<()> {
        // SAFETY: we've already allocated enough space
        unsafe {
            data.set_len(capacity);
        }
        data.fill(*template);
        Ok(())
    }
}

// SIMD optimization for primitive types
#[cfg(target_arch = "x86_64")]
impl MergeBuffer<i32> {
    /// Initialize the buffer using SIMD operations for i32.
    #[inline]
    fn initialize_buffer(data: &mut Vec<i32>, capacity: usize, template: &i32) -> Result<()> {
        use std::arch::x86_64::*;
        
        // SAFETY: we've already allocated enough space
        unsafe {
            data.set_len(capacity);
            
            if is_x86_feature_detected!("avx2") {
                let value = _mm256_set1_epi32(*template);
                let ptr = data.as_mut_ptr() as *mut __m256i;
                let chunks = capacity / 8;
                
                for i in 0..chunks {
                    _mm256_store_si256(ptr.add(i), value);
                }
                
                // Fill remaining elements
                let remaining = capacity % 8;
                if remaining > 0 {
                    let start = chunks * 8;
                    data[start..].fill(*template);
                }
            } else {
                // Fall back to regular fill
                data.fill(*template);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod buffer_tests {
    use super::*;
    use std::alloc::Layout;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn test_buffer_allocation() {
        let buffer = MergeBuffer::new(1000, &42i32).unwrap();
        assert_eq!(buffer.capacity(), 1000);
        assert!(!buffer.is_empty());
        assert!(buffer.as_slice().iter().all(|&x| x == 42));
    }

    #[test]
    fn test_buffer_zero_capacity() {
        let buffer = MergeBuffer::<i32>::new(0, &42).unwrap();
        assert_eq!(buffer.capacity(), 0);
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_buffer_huge_allocation() {
        let result = MergeBuffer::<i32>::new(usize::MAX / 4, &42);
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
    fn test_buffer_copy_type() {
        let start = std::time::Instant::now();
        let buffer = MergeBuffer::new(1_000_000, &42i32).unwrap();
        let duration = start.elapsed();
        
        // Verify correctness
        assert_eq!(buffer.capacity(), 1_000_000);
        assert!(buffer.as_slice().iter().all(|&x| x == 42));
        
        // Should be fast due to memcpy
        assert!(duration.as_micros() < 1000, "Copy initialization took too long");
    }

    #[test]
    fn test_buffer_clone_type() {
        #[derive(Debug, Clone, PartialEq)]
        struct NonCopy(i32);

        let template = NonCopy(42);
        let buffer = MergeBuffer::new(100, &template).unwrap();
        
        assert_eq!(buffer.capacity(), 100);
        assert!(buffer.as_slice().iter().all(|x| x == &template));
    }

    #[cfg(target_arch = "x86_64")]
    mod simd_tests {
        use super::*;

        #[test]
        fn test_buffer_simd_i32() {
            if !is_x86_feature_detected!("avx2") {
                return;
            }

            let start = std::time::Instant::now();
            let buffer = MergeBuffer::new(1_000_000, &42i32).unwrap();
            let duration = start.elapsed();
            
            assert_eq!(buffer.capacity(), 1_000_000);
            assert!(buffer.as_slice().iter().all(|&x| x == 42));
            assert!(duration.as_micros() < 500, "SIMD initialization took too long");
        }

        #[test]
        fn test_buffer_simd_f32() {
            if !is_x86_feature_detected!("avx") {
                return;
            }

            let start = std::time::Instant::now();
            let buffer = MergeBuffer::new(1_000_000, &42.0f32).unwrap();
            let duration = start.elapsed();
            
            assert_eq!(buffer.capacity(), 1_000_000);
            assert!(buffer.as_slice().iter().all(|&x| (x - 42.0).abs() < f32::EPSILON));
            assert!(duration.as_micros() < 500, "SIMD initialization took too long");
        }

        #[test]
        fn test_buffer_simd_alignment() {
            if !is_x86_feature_detected!("avx2") {
                return;
            }

            let buffer = MergeBuffer::new(1000, &42i32).unwrap();
            let ptr = buffer.as_slice().as_ptr();
            
            // AVX2 requires 32-byte alignment for optimal performance
            assert_eq!(ptr as usize % 32, 0, "Buffer not properly aligned for SIMD");
        }

        #[test]
        fn test_buffer_simd_error_conditions() {
            if !is_x86_feature_detected!("avx2") {
                return;
            }

            // Test with size that's not multiple of SIMD width
            let buffer = MergeBuffer::new(1001, &42i32).unwrap();
            assert_eq!(buffer.capacity(), 1001);
            assert!(buffer.as_slice().iter().all(|&x| x == 42));
        }
    }

    #[test]
    fn test_buffer_thread_safety() {
        let size = 1000;
        let buffer = Arc::new(MergeBuffer::new(size, &42i32).unwrap());
        let threads = 4;
        let barrier = Arc::new(Barrier::new(threads));
        
        let handles: Vec<_> = (0..threads)
            .map(|i| {
                let buffer = buffer.clone();
                let barrier = barrier.clone();
                let chunk_size = size / threads;
                let start = i * chunk_size;
                
                thread::spawn(move || {
                    barrier.wait();
                    let slice = &buffer.as_slice()[start..start + chunk_size];
                    assert!(slice.iter().all(|&x| x == 42));
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_buffer_reallocation() {
        struct ReallocCounter {
            reallocs: AtomicUsize,
        }

        impl ReallocCounter {
            const fn new() -> Self {
                Self {
                    reallocs: AtomicUsize::new(0),
                }
            }
        }

        unsafe impl GlobalAlloc for ReallocCounter {
            unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
                std::alloc::System.alloc(layout)
            }

            unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
                self.reallocs.fetch_add(1, Ordering::SeqCst);
                std::alloc::System.dealloc(ptr, layout)
            }
        }

        #[global_allocator]
        static REALLOC_COUNTER: ReallocCounter = ReallocCounter::new();

        // Reset counter
        REALLOC_COUNTER.reallocs.store(0, Ordering::SeqCst);

        // Create and drop buffer
        {
            let buffer = MergeBuffer::new(1000, &42i32).unwrap();
            assert_eq!(buffer.capacity(), 1000);
        }

        // Should only have one deallocation
        assert_eq!(REALLOC_COUNTER.reallocs.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_buffer_drop() {
        static DROP_COUNT: AtomicUsize = AtomicUsize::new(0);
        
        #[derive(Clone)]
        struct DropCounter;
        impl Drop for DropCounter {
            fn drop(&mut self) {
                DROP_COUNT.fetch_add(1, Ordering::SeqCst);
            }
        }

        // Reset counter
        DROP_COUNT.store(0, Ordering::SeqCst);

        // Create and drop buffer
        {
            let size = 100;
            let _buffer = MergeBuffer::new(size, &DropCounter).unwrap();
        }

        // Each element should be dropped exactly once
        assert_eq!(DROP_COUNT.load(Ordering::SeqCst), 100);
    }
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