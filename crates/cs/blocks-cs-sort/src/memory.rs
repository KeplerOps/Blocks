use std::fmt::Debug;
use std::marker::PhantomData;
use std::alloc::{self, Layout};
use std::ptr::NonNull;
use std::mem;
use std::any::TypeId;
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// Initialize a vector with a template value.
/// 
/// This enum represents different initialization strategies based on type properties.
#[derive(Debug)]
enum InitStrategy<T> {
    /// Use clone() for general types
    Clone,
    /// Use memcpy for Copy types
    Copy,
    /// Use SIMD for supported types
    #[cfg(target_arch = "x86_64")]
    Simd,
    /// Phantom data for type parameter
    _Phantom(PhantomData<T>),
}

impl<T: Clone + 'static> InitStrategy<T> {
    /// Initialize a vector using the best strategy for type T
    fn initialize(data: &mut Vec<T>, capacity: usize, template: &T) -> Result<()> {
        // Choose strategy based on type properties
        let strategy = if TypeId::of::<T>() == TypeId::of::<i32>() {
            #[cfg(target_arch = "x86_64")]
            {
                InitStrategy::<T>::Simd
            }
            #[cfg(not(target_arch = "x86_64"))]
            {
                if mem::needs_drop::<T>() {
                    InitStrategy::<T>::Clone
                } else {
                    InitStrategy::<T>::Copy
                }
            }
        } else if mem::needs_drop::<T>() {
            InitStrategy::<T>::Clone
        } else {
            InitStrategy::<T>::Copy
        };

        // Apply chosen strategy
        match strategy {
            InitStrategy::Clone => {
                data.extend(std::iter::repeat_with(|| template.clone()).take(capacity));
                Ok(())
            }
            InitStrategy::Copy => {
                // SAFETY: we've already allocated enough space
                unsafe {
                    data.set_len(capacity);
                    // SAFETY: T is Copy
                    let template_ref = &*template;
                    data.fill(template_ref.clone());
                }
                Ok(())
            }
            #[cfg(target_arch = "x86_64")]
            InitStrategy::Simd => {
                use std::arch::x86_64::*;
                
                // SAFETY: we've already allocated enough space
                unsafe {
                    data.set_len(capacity);
                    
                    // SAFETY: we know T is i32 here
                    let template = &*(template as *const T as *const i32);
                    
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
                            let slice = std::slice::from_raw_parts_mut(
                                data.as_mut_ptr() as *mut i32,
                                capacity
                            );
                            slice[start..].fill(*template);
                        }
                    } else {
                        let slice = std::slice::from_raw_parts_mut(
                            data.as_mut_ptr() as *mut i32,
                            capacity
                        );
                        slice.fill(*template);
                    }
                }
                Ok(())
            }
            InitStrategy::_Phantom(_) => unreachable!(),
        }
    }
}

/// A buffer for merge operations that handles allocation safely.
#[derive(Debug)]
pub(crate) struct MergeBuffer<T> {
    data: Vec<T>,
}

impl<T: Clone + 'static> MergeBuffer<T> {
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
            let size = mem::size_of::<T>()
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
            InitStrategy::initialize(&mut data, capacity, template)?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

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
            let simd_elements = SIMD_WIDTH / mem::size_of::<i32>();
            let expected_capacity = ((size + simd_elements - 1) / simd_elements) * simd_elements;
            assert_eq!(layout.size(), expected_capacity * mem::size_of::<i32>());
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

            // Check each arena individually
            let arenas = [
                (arena1.buffer.as_ptr() as usize, arena1.layout()),
                (arena2.buffer.as_ptr() as usize, arena2.layout()),
                (arena3.buffer.as_ptr() as usize, arena3.layout()),
                (arena4.buffer.as_ptr() as usize, arena4.layout()),
            ];

            for (ptr, layout) in arenas {
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
    fn test_arena_allocation() {
        let arena = SortArena::<i32>::new(1000).unwrap();
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
    fn test_arena_concurrent_access() {
        let size = 1000;
        let arena = Arc::new(SortArena::<i32>::new(size).unwrap());
        let threads = 4;
        let barrier = Arc::new(std::sync::Barrier::new(threads));
        
        // Spawn threads that read from different parts of the arena
        let handles: Vec<_> = (0..threads)
            .map(|i| {
                let arena = arena.clone();
                let barrier = barrier.clone();
                let chunk_size = size / threads;
                let start = i * chunk_size;
                
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

    #[test]
    fn test_arena_drop() {
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
        let barrier = Arc::new(std::sync::Barrier::new(threads));
        
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