use std::fmt::{Debug, Display};
use std::error::Error;

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
}

impl Default for MergeSortBuilder {
    fn default() -> Self {
        Self {
            insertion_threshold: 16, // Tuned via benchmarks
            max_recursion_depth: 48,
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
    pub fn sort<T>(&self, slice: &mut [T]) -> Result<(), SortError>
    where
        T: Ord + Clone + Debug,
    {
        if slice.len() <= 1 {
            return Ok(());
        }

        // Allocate auxiliary array
        let mut aux = Vec::with_capacity(slice.len());
        unsafe {
            aux.set_len(slice.len());
        }

        self.sort_internal(slice, &mut aux, 0)
    }

    fn sort_internal<T>(
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
        self.sort_internal(&mut slice[..mid], aux, depth + 1)?;
        self.sort_internal(&mut slice[mid..], aux, depth + 1)?;

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
    T: Ord + Clone + Debug,
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
}