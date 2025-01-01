use std::fmt::{Debug, Display};
use std::error::Error;
use rayon;

/// Error types that can occur during sorting operations
#[derive(Debug, Clone)]
pub struct SortError {
    kind: SortErrorKind,
    message: String,
}

impl Display for SortError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind, self.message)
    }
}

impl Error for SortError {}

/// Specific types of errors that can occur during sorting
#[derive(Debug, Clone)]
pub enum SortErrorKind {
    /// The recursion depth exceeded the configured maximum
    RecursionLimitExceeded,
    /// Memory allocation failed
    AllocationFailed,
}

impl Display for SortErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortErrorKind::RecursionLimitExceeded => write!(f, "Recursion limit exceeded"),
            SortErrorKind::AllocationFailed => write!(f, "Memory allocation failed"),
        }
    }
}

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
            parallel_threshold: 1024, // Arrays larger than this will be sorted in parallel
        }
    }
}

impl MergeSortBuilder {
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

    /// Sorts a mutable slice using the configured settings
    /// 
    /// # Errors
    /// 
    /// Returns `SortError` if:
    /// - Memory allocation fails
    /// - Maximum recursion depth is exceeded
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

    pub fn sort<T>(&self, slice: &mut [T]) -> Result<(), SortError>
    where
        T: Ord + Clone + Debug + Send + Sync,
    {
        if slice.len() <= 1 {
            return Ok(());
        }

        // Allocate auxiliary array
        let mut aux = Vec::with_capacity(slice.len());
        unsafe {
            aux.set_len(slice.len());
        }

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
    ) -> Result<(), SortError>
    where
        T: Ord + Clone + Debug,
    {
        // Check recursion depth
        if depth >= self.max_recursion_depth {
            return Err(SortError {
                kind: SortErrorKind::RecursionLimitExceeded,
                message: format!(
                    "Exceeded maximum recursion depth of {}",
                    self.max_recursion_depth
                ),
            });
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
    ) -> Result<(), SortError>
    where
        T: Ord + Clone + Debug + Send + Sync,
    {
        if depth >= self.max_recursion_depth {
            return Err(SortError {
                kind: SortErrorKind::RecursionLimitExceeded,
                message: format!(
                    "Exceeded maximum recursion depth of {}",
                    self.max_recursion_depth
                ),
            });
        }

        if slice.len() <= self.insertion_threshold {
            insertion_sort(slice);
            return Ok(());
        }

        let mid = slice.len() / 2;

        // Sort halves in parallel
        if slice.len() >= self.parallel_threshold {
            // Create a clone of the slice to allow parallel processing
            let mut temp = slice.to_vec();
            let (left, right) = temp.split_at_mut(mid);

            // Sort halves in parallel
            rayon::join(
                || self.sort_sequential(left, aux, depth + 1),
                || self.sort_sequential(right, aux, depth + 1),
            );

            // Copy back the sorted results
            slice.copy_from_slice(&temp);
        } else {
            // Fall back to sequential for smaller chunks
            self.sort_sequential(&mut slice[..mid], aux, depth + 1)?;
            self.sort_sequential(&mut slice[mid..], aux, depth + 1)?;
        }

        // Merge the sorted halves
        merge(slice, mid, aux);
        Ok(())
    }
}

/// Sorts a slice using merge sort with default settings
/// 
/// This is a convenience wrapper around `MergeSortBuilder`.
/// For more control, use `MergeSortBuilder` directly.
pub fn sort<T>(slice: &mut [T]) -> Result<(), SortError>
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

fn merge<T: Ord + Clone>(slice: &mut [T], mid: usize, aux: &mut Vec<T>) {
    // Copy to auxiliary array
    aux[..slice.len()].clone_from_slice(slice);

    let (left, right) = aux[..slice.len()].split_at(mid);
    
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
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

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
    fn test_recursion_limit() {
        let mut arr: Vec<i32> = (0..1_000_000).collect();
        let result = MergeSortBuilder::new()
            .max_recursion_depth(3)
            .sort(&mut arr);

        assert!(matches!(
            result,
            Err(SortError {
                kind: SortErrorKind::RecursionLimitExceeded,
                ..
            })
        ));
    }

    #[test]
    fn test_custom_threshold() {
        let mut arr = vec![5, 2, 8, 1, 9, 3];
        MergeSortBuilder::new()
            .insertion_threshold(2)
            .sort(&mut arr)
            .unwrap();
        assert_eq!(arr, vec![1, 2, 3, 5, 8, 9]);
    }

    #[test]
    fn test_parallel_sorting() {
        // Create a large array to ensure parallel sorting is used
        let size = 100_000;
        let mut arr: Vec<i32> = (0..size).rev().collect();
        let mut expected = arr.clone();
        expected.sort();

        // Count parallel executions
        let parallel_count = Arc::new(AtomicUsize::new(0));
        let parallel_count_clone = Arc::clone(&parallel_count);

        rayon::ThreadPoolBuilder::new()
            .start_handler(move |_| {
                parallel_count_clone.fetch_add(1, Ordering::SeqCst);
            })
            .build_global()
            .unwrap();

        MergeSortBuilder::new()
            .parallel(true)
            .parallel_threshold(1000)
            .sort(&mut arr)
            .unwrap();

        assert_eq!(arr, expected);
        assert!(parallel_count.load(Ordering::SeqCst) > 0);
    }

    #[test]
    fn test_parallel_threshold() {
        let size = 10_000;
        let mut arr: Vec<i32> = (0..size).rev().collect();
        
        // Set threshold higher than array size - should use sequential sort
        let mut arr1 = arr.clone();
        MergeSortBuilder::new()
            .parallel(true)
            .parallel_threshold(size * 2)
            .sort(&mut arr1)
            .unwrap();

        // Set threshold lower than array size - should use parallel sort
        let mut arr2 = arr.clone();
        MergeSortBuilder::new()
            .parallel(true)
            .parallel_threshold(size / 2)
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
}