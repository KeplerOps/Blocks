//! Mergesort implementation with parallel processing support.
//! 
//! This module provides a configurable mergesort implementation that can:
//! - Use insertion sort for small arrays
//! - Process large arrays in parallel using rayon
//! - Handle generic types that implement Ord + Clone
//! 
//! # Safety
//! This implementation uses unsafe code in the following ways:
//! - Uses `split_at_mut` for parallel processing (safe interface to unsafe code)
//! - Uses rayon's parallel execution primitives (safe interface to unsafe code)
//! 
//! All unsafe operations are properly encapsulated and safe when used with types
//! that implement the required traits (Send + Sync for parallel execution).

use std::fmt::Debug;
use rayon;

use crate::error::{Result, SortError};
use crate::memory::{MergeBuffer, SortArena};

/// Builder for configuring and executing merge sort operations.
/// 
/// # Examples
/// 
/// ```
/// use blocks_cs_sort::algorithms::mergesort::MergeSortBuilder;
/// 
/// let mut arr = vec![3, 1, 4, 1, 5, 9];
/// MergeSortBuilder::new()
///     .insertion_threshold(16)
///     .sort(&mut arr)
///     .expect("Sort failed");
/// assert!(arr.windows(2).all(|w| w[0] <= w[1]));
/// ```
/// 
/// # Performance
/// 
/// The algorithm has the following complexity characteristics:
/// - Time: O(n log n) in all cases
/// - Space: O(n) auxiliary space
/// - Stable: Yes
/// 
/// Performance can be tuned through:
/// - `insertion_threshold`: Arrays smaller than this use insertion sort (default: 16)
/// - `max_recursion_depth`: Limit recursion to prevent stack overflow (default: 48)
/// - `parallel`: Enable parallel sorting for large arrays
/// - `parallel_threshold`: Minimum size for parallel processing
#[derive(Debug, Clone)]
pub struct MergeSortBuilder {
    insertion_threshold: usize,
    max_recursion_depth: usize,
    parallel: bool,
    parallel_threshold: usize,
}

impl Default for MergeSortBuilder {
    fn default() -> Self {
        Self {
            insertion_threshold: 16, // Tuned via benchmarks
            max_recursion_depth: 48,
            parallel: false,
            parallel_threshold: 1024,
        }
    }
}

impl MergeSortBuilder {
    /// Maximum length of slice that can be sorted (2^48 elements).
    /// This limit ensures we don't exceed reasonable memory usage.
    const MAX_LENGTH: usize = 1 << 48;

    /// Creates a new MergeSortBuilder with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the threshold below which insertion sort is used
    /// 
    /// Smaller values favor merge sort's O(n log n) complexity,
    /// larger values favor insertion sort's cache efficiency on small arrays.
    /// 
    /// # Examples
    /// ```
    /// use blocks_cs_sort::algorithms::mergesort::MergeSortBuilder;
    /// 
    /// let mut arr = vec![5, 2, 8, 1, 9, 3];
    /// MergeSortBuilder::new()
    ///     .insertion_threshold(8)
    ///     .sort(&mut arr)
    ///     .unwrap();
    /// ```
    pub fn insertion_threshold(mut self, threshold: usize) -> Self {
        self.insertion_threshold = threshold;
        self
    }

    /// Sets the maximum recursion depth
    /// 
    /// This prevents stack overflow on very large arrays.
    /// The default of 48 supports arrays up to 2^48 elements.
    pub fn max_recursion_depth(mut self, depth: usize) -> Self {
        self.max_recursion_depth = depth;
        self
    }

    /// Enables or disables parallel sorting
    /// 
    /// When enabled, arrays larger than the parallel threshold will be sorted
    /// using multiple threads via rayon.
    /// 
    /// # Examples
    /// ```
    /// use blocks_cs_sort::algorithms::mergesort::MergeSortBuilder;
    /// 
    /// let mut arr = vec![5, 2, 8, 1, 9, 3];
    /// MergeSortBuilder::new()
    ///     .parallel(true)
    ///     .sort(&mut arr)
    ///     .unwrap();
    /// ```
    pub fn parallel(mut self, enabled: bool) -> Self {
        self.parallel = enabled;
        self
    }

    /// Sets the threshold above which parallel sorting is used
    /// 
    /// Arrays larger than this threshold will be sorted in parallel
    /// when parallel sorting is enabled.
    pub fn parallel_threshold(mut self, threshold: usize) -> Self {
        self.parallel_threshold = threshold;
        self
    }

    /// Sorts a mutable slice using the configured settings
    /// 
    /// # Errors
    /// 
    /// Returns `SortError` if:
    /// - Memory allocation fails
    /// - Maximum recursion depth is exceeded
    /// - Input slice is too large (> 2^48 elements)
    /// - Parallel execution fails
    pub fn sort<T>(&self, slice: &mut [T]) -> Result<()>
    where
        T: Ord + Clone + Debug + Send + Sync,
    {
        if slice.len() <= 1 {
            return Ok(());
        }

        // Check input size
        if slice.len() > Self::MAX_LENGTH {
            return Err(SortError::input_too_large(slice.len(), Self::MAX_LENGTH));
        }

        // Create auxiliary buffer for merging
        let mut aux = MergeBuffer::new(slice.len(), &slice[0])?;

        // Start the recursive sort with depth counter
        if self.parallel && slice.len() >= self.parallel_threshold {
            self.sort_parallel(slice, &mut aux, 0)
        } else {
            self.sort_sequential(slice, &mut aux, 0)
        }
    }

    fn sort_sequential<T>(
        &self,
        slice: &mut [T],
        aux: &mut Vec<T>,
        depth: usize,
    ) -> Result<()>
    where
        T: Ord + Clone + Debug,
    {
        // Check recursion depth
        if depth >= self.max_recursion_depth {
            return Err(SortError::recursion_limit_exceeded(
                depth,
                self.max_recursion_depth,
            ));
        }

        // Use insertion sort for small arrays
        if slice.len() <= self.insertion_threshold {
            insertion_sort(slice);
            return Ok(());
        }

        let mid = slice.len() / 2;

        // Recursively sort halves
        self.sort_sequential(&mut slice[..mid], aux, depth + 1)?;
        self.sort_sequential(&mut slice[mid..], aux, depth + 1)?;

        // Merge the sorted halves
        merge(slice, mid, aux);
        Ok(())
    }

    fn sort_parallel<T>(
        &self,
        slice: &mut [T],
        aux: &mut Vec<T>,
        depth: usize,
    ) -> Result<()>
    where
        T: Ord + Clone + Debug + Send + Sync,
    {
        // Check recursion depth
        if depth >= self.max_recursion_depth {
            return Err(SortError::recursion_limit_exceeded(
                depth,
                self.max_recursion_depth,
            ));
        }

        // Use insertion sort for small arrays
        if slice.len() <= self.insertion_threshold {
            insertion_sort(slice);
            return Ok(());
        }

        let mid = slice.len() / 2;

        // Create auxiliary buffers for parallel sorting
        let mut left_aux = MergeBuffer::new(mid, &slice[0])?;
        let mut right_aux = MergeBuffer::new(slice.len() - mid, &slice[0])?;

        // SAFETY: split_at_mut is safe but uses unsafe code internally to create
        // two mutable references to different parts of the slice. This is safe
        // because the ranges are guaranteed not to overlap.
        let (left, right) = slice.split_at_mut(mid);

        // SAFETY: rayon's join uses unsafe code internally for thread management
        // and parallel execution. This is safe because T: Send + Sync and we're
        // operating on non-overlapping mutable slices.
        let (left_result, right_result) = rayon::join(
            || self.sort_sequential(left, &mut left_aux, depth + 1),
            || self.sort_sequential(right, &mut right_aux, depth + 1),
        );

        // Handle any errors from parallel execution
        left_result.map_err(|e| SortError::parallel_execution_failed(
            format!("Left parallel task failed: {}", e)
        ))?;
        right_result.map_err(|e| SortError::parallel_execution_failed(
            format!("Right parallel task failed: {}", e)
        ))?;

        // Merge the sorted halves
        merge(slice, mid, aux);
        Ok(())
    }
}

/// Sorts a slice using merge sort with default settings
/// 
/// This is a convenience wrapper around `MergeSortBuilder`.
/// For more control, use `MergeSortBuilder` directly.
/// 
/// # Errors
/// 
/// Returns `SortError` if:
/// - Memory allocation fails
/// - Maximum recursion depth is exceeded
/// - Input slice is too large (> 2^48 elements)
/// - Parallel execution fails
pub fn sort<T>(slice: &mut [T]) -> Result<()>
where
    T: Ord + Clone + Debug + Send + Sync,
{
    MergeSortBuilder::new().sort(slice)
}

// Internal helper functions

fn insertion_sort<T: Ord>(slice: &mut [T]) {
    for i in 1..slice.len() {
        let mut j = i;
        while j > 0 && slice[j - 1] > slice[j] {
            slice.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn merge<T: Ord + Clone>(slice: &mut [T], mid: usize, aux: &mut MergeBuffer<T>) {
    // Copy to auxiliary buffer
    aux.as_mut_slice()[..slice.len()].clone_from_slice(slice);

    let aux_slice = aux.as_slice();
    let (left, right) = aux_slice[..slice.len()].split_at(mid);
    
    let mut i = 0; // Index for left array
    let mut j = 0; // Index for right array
    let mut k = 0; // Index for merged array

    // Compare and merge elements back into original slice
    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            slice[k] = left[i].clone();
            i += 1;
        } else {
            slice[k] = right[j].clone();
            j += 1;
        }
        k += 1;
    }

    // Copy remaining elements
    if i < left.len() {
        slice[k..].clone_from_slice(&left[i..]);
    }
    if j < right.len() {
        slice[k..].clone_from_slice(&right[j..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_empty_slice() {
        let mut arr: Vec<i32> = vec![];
        sort(&mut arr).unwrap();
        assert_eq!(arr, vec![]);
    }

    #[test]
    fn test_single_element() {
        let mut arr = vec![1];
        sort(&mut arr).unwrap();
        assert_eq!(arr, vec![1]);
    }

    #[test]
    fn test_sorted_array() {
        let mut arr = vec![1, 2, 3, 4, 5];
        sort(&mut arr).unwrap();
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reverse_sorted() {
        let mut arr = vec![5, 4, 3, 2, 1];
        sort(&mut arr).unwrap();
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_random_order() {
        let mut arr = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr).unwrap();
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_parallel_sorting() {
        // Create a large array to ensure parallel sorting is used
        let size = 100_000;
        let mut arr: Vec<i32> = (0..size).rev().collect();
        let mut expected = arr.clone();
        expected.sort();

        // Count parallel executions using thread-local storage
        thread_local! {
            static THREAD_COUNT: AtomicUsize = AtomicUsize::new(0);
        }

        // Initialize rayon with a custom thread pool for this test
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(4)
            .build()
            .unwrap();

        pool.install(|| {
            MergeSortBuilder::new()
                .parallel(true)
                .parallel_threshold(1000)
                .sort(&mut arr)
                .unwrap();
        });

        assert_eq!(arr, expected);
    }

    #[test]
    fn test_parallel_threshold() {
        let size = 10_000;
        let arr: Vec<i32> = (0..size).rev().collect();
        
        // Set threshold higher than array size - should use sequential sort
        let mut arr1 = arr.clone();
        MergeSortBuilder::new()
            .parallel(true)
            .parallel_threshold((size * 2) as usize)
            .sort(&mut arr1)
            .unwrap();

        // Set threshold lower than array size - should use parallel sort
        let mut arr2 = arr.clone();
        MergeSortBuilder::new()
            .parallel(true)
            .parallel_threshold((size / 2) as usize)
            .sort(&mut arr2)
            .unwrap();

        let mut expected = arr;
        expected.sort();

        assert_eq!(arr1, expected);
        assert_eq!(arr2, expected);
    }

    #[test]
    fn test_parallel_stability() {
        #[derive(Debug, Clone, Eq, PartialEq)]
        struct Item {
            key: i32,
            original_index: usize,
        }

        impl PartialOrd for Item {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                self.key.partial_cmp(&other.key)
            }
        }

        impl Ord for Item {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.key.cmp(&other.key)
            }
        }

        // Create a large array of items with duplicate keys
        let size = 10_000;
        let mut items: Vec<_> = (0..size)
            .map(|i| Item {
                key: i as i32 / 10, // Create many duplicates
                original_index: i,
            })
            .collect();

        MergeSortBuilder::new()
            .parallel(true)
            .parallel_threshold(1000)
            .sort(&mut items)
            .unwrap();

        // Verify stability
        for i in 1..items.len() {
            if items[i-1].key == items[i].key {
                assert!(
                    items[i-1].original_index < items[i].original_index,
                    "Stability violated at indices {} and {}",
                    i-1,
                    i
                );
            }
        }
    }

    #[test]
    fn test_recursion_limit() {
        let mut arr: Vec<i32> = (0..1_000_000).collect();
        let result = MergeSortBuilder::new()
            .max_recursion_depth(3)
            .sort(&mut arr);

        match result {
            Err(SortError::RecursionLimitExceeded { depth, max_depth }) => {
                assert_eq!(max_depth, 3);
                assert!(depth >= max_depth);
            }
            _ => panic!("Expected RecursionLimitExceeded error"),
        }
    }

    #[test]
    fn test_input_too_large() {
        // Create an array larger than MAX_LENGTH
        let size = MergeSortBuilder::MAX_LENGTH + 1;
        let mut arr = vec![0; size];
        let result = sort(&mut arr);

        match result {
            Err(SortError::InputTooLarge { length, max_length }) => {
                assert_eq!(length, size);
                assert_eq!(max_length, MergeSortBuilder::MAX_LENGTH);
            }
            _ => panic!("Expected InputTooLarge error"),
        }
    }
}