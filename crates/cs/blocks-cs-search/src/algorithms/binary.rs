#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

use std::cmp::Ordering;

/// Performs a binary search on a sorted slice to find a target element.
///
/// This function implements an iterative binary search algorithm that efficiently
/// finds an element in a sorted slice. It's particularly useful when:
/// - The data is already sorted
/// - The dataset is large
/// - Multiple searches need to be performed on the same data
/// - Performance is critical
///
/// # Algorithm Details
/// - Time Complexity: O(log n) where n is the length of the slice
/// - Space Complexity: O(1)
/// - In-place: Yes
/// - Stable: N/A (search only)
/// - Early exit: Yes (returns as soon as element is found)
///
/// # Type Parameters
/// - `T`: The type of elements in the slice, must implement [`Ord`]
///
/// # Arguments
/// * `arr` - A sorted slice of elements to search through
/// * `target` - The element to find in the slice
///
/// # Returns
/// - `Some(index)` if the element is found at position `index`
/// - `None` if the element is not present in the slice
///
/// # Panics
/// This function does not panic, but the behavior is undefined if the slice is not sorted.
///
/// # Examples
/// Basic usage with integers:
/// ```
/// use blocks_cs_search::algorithms::binary::search;
/// let numbers = vec![1, 3, 4, 6, 8, 9, 11];
/// assert_eq!(search(&numbers, &6), Some(3));
/// assert_eq!(search(&numbers, &7), None);
/// ```
///
/// Works with any type that implements [`Ord`]:
/// ```
/// use blocks_cs_search::algorithms::binary::search;
/// let words = vec!["apple", "banana", "cherry", "date"];
/// assert_eq!(search(&words, &"cherry"), Some(2));
/// ```
///
/// # Performance Considerations
/// - Binary search requires the input to be sorted
/// - For small datasets (n < 50), consider using [`linear_search`] instead
/// - If many insertions/deletions occur between searches, consider using a [`BTreeMap`] or [`BTreeSet`]
/// - For frequently modified datasets with frequent searches, consider using a balanced tree structure
///
/// # Implementation Notes
/// - Uses an iterative approach to avoid stack overhead
/// - Handles integer overflow correctly
/// - Optimized for cache-friendliness with linear memory access pattern
///
/// [`linear_search`]: super::linear::search
/// [`BTreeMap`]: std::collections::BTreeMap
/// [`BTreeSet`]: std::collections::BTreeSet
#[inline]
pub fn search<T: Ord>(arr: &[T], target: &T) -> Option<usize> {
    if arr.is_empty() {
        return None;
    }

    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2; // Avoid potential integer overflow
        
        match arr[mid].cmp(target) {
            Ordering::Equal => return Some(mid),
            Ordering::Greater => right = mid,
            Ordering::Less => left = mid + 1,
        }
    }

    None
}

/// Performs a binary search on a sorted slice to find the insertion point for a target element.
///
/// This function returns either the index of the matching element or the index where the
/// target element should be inserted to maintain sorted order.
///
/// # Algorithm Details
/// Same performance characteristics as [`search`], but provides additional information
/// about where an element should be inserted if not found.
///
/// # Type Parameters
/// - `T`: The type of elements in the slice, must implement [`Ord`]
///
/// # Arguments
/// * `arr` - A sorted slice of elements to search through
/// * `target` - The element to find the insertion point for
///
/// # Returns
/// - `Ok(index)` if the element is found at position `index`
/// - `Err(index)` if the element is not found, where `index` is where it should be inserted
///
/// # Examples
/// Finding insertion points:
/// ```
/// use blocks_cs_search::algorithms::binary::search_insert;
/// let numbers = vec![1, 3, 5, 7];
/// assert_eq!(search_insert(&numbers, &4), Err(2)); // 4 should go between 3 and 5
/// assert_eq!(search_insert(&numbers, &5), Ok(2));  // 5 is found at index 2
/// ```
///
/// # Use Cases
/// - Maintaining sorted collections
/// - Implementing insertion sort
/// - Finding bounds in sorted data
/// - Implementing binary search trees
#[inline]
pub fn search_insert<T: Ord>(arr: &[T], target: &T) -> Result<usize, usize> {
    if arr.is_empty() {
        return Err(0);
    }

    let mut left = 0;
    let mut right = arr.len();

    while left < right {
        let mid = left + (right - left) / 2;
        
        match arr[mid].cmp(target) {
            Ordering::Equal => return Ok(mid),
            Ordering::Greater => right = mid,
            Ordering::Less => left = mid + 1,
        }
    }

    Err(left)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;
    use std::cmp::Ordering;

    // Property-based tests
    #[quickcheck]
    fn prop_found_element_exists(mut xs: Vec<i32>) -> bool {
        if xs.is_empty() {
            return true;
        }
        xs.sort_unstable();
        let target = xs[xs.len() / 2]; // Pick middle element to avoid edge cases
        search(&xs, &target).is_some()
    }

    #[quickcheck]
    fn prop_maintains_invariants(mut xs: Vec<i32>, x: i32) -> bool {
        xs.sort_unstable();
        match search(&xs, &x) {
            Some(i) => i < xs.len() && xs[i] == x,
            None => !xs.contains(&x)
        }
    }

    #[quickcheck]
    fn prop_insert_point_valid(mut xs: Vec<i32>, x: i32) -> bool {
        xs.sort_unstable();
        match search_insert(&xs, &x) {
            Ok(i) => i < xs.len() && xs[i] == x,
            Err(i) => {
                i <= xs.len() && 
                (i == 0 || xs[i-1] < x) &&
                (i == xs.len() || xs[i] > x)
            }
        }
    }

    #[quickcheck]
    fn prop_search_equivalent_to_binary_search(mut xs: Vec<i32>, x: i32) -> bool {
        xs.sort_unstable();
        search(&xs, &x).map(Ok).unwrap_or_else(|| Err(0)) == xs.binary_search(&x)
    }

    // Unit tests for specific cases
    #[test]
    fn test_empty_array() {
        let arr: Vec<i32> = vec![];
        assert_eq!(search(&arr, &1), None);
        assert_eq!(search_insert(&arr, &1), Err(0));
    }

    #[test]
    fn test_single_element() {
        let arr = vec![1];
        assert_eq!(search(&arr, &1), Some(0));
        assert_eq!(search(&arr, &0), None);
        assert_eq!(search(&arr, &2), None);
        
        assert_eq!(search_insert(&arr, &1), Ok(0));
        assert_eq!(search_insert(&arr, &0), Err(0));
        assert_eq!(search_insert(&arr, &2), Err(1));
    }

    #[test]
    fn test_multiple_elements() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(search(&arr, &5), Some(2));
        assert_eq!(search(&arr, &6), None);
        
        assert_eq!(search_insert(&arr, &5), Ok(2));
        assert_eq!(search_insert(&arr, &6), Err(3));
    }

    #[test]
    fn test_first_last_elements() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(search(&arr, &1), Some(0));
        assert_eq!(search(&arr, &9), Some(4));
        
        assert_eq!(search_insert(&arr, &0), Err(0));
        assert_eq!(search_insert(&arr, &10), Err(5));
    }

    #[test]
    fn test_strings() {
        let arr = vec!["apple", "banana", "cherry", "date"];
        assert_eq!(search(&arr, &"cherry"), Some(2));
        assert_eq!(search(&arr, &"blueberry"), None);
        
        assert_eq!(search_insert(&arr, &"cherry"), Ok(2));
        assert_eq!(search_insert(&arr, &"blueberry"), Err(1));
    }

    #[test]
    fn test_custom_type() {
        #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
        struct Version(u32, u32, u32);

        let arr = vec![
            Version(1, 0, 0),
            Version(1, 1, 0),
            Version(2, 0, 0),
            Version(2, 1, 0),
        ];

        assert_eq!(search(&arr, &Version(2, 0, 0)), Some(2));
        assert_eq!(search(&arr, &Version(1, 2, 0)), None);
        
        assert_eq!(search_insert(&arr, &Version(2, 0, 0)), Ok(2));
        assert_eq!(search_insert(&arr, &Version(1, 2, 0)), Err(2));
    }

    // Edge cases
    #[test]
    fn test_repeated_elements() {
        let arr = vec![1, 2, 2, 2, 3];
        // Should find one of the 2s
        let idx = search(&arr, &2).unwrap();
        assert!(idx >= 1 && idx <= 3);
        assert_eq!(arr[idx], 2);
    }

    #[test]
    fn test_large_array() {
        let arr: Vec<i32> = (0..10000).map(|x| x * 2).collect();
        assert_eq!(search(&arr, &5000), Some(2500));
        assert_eq!(search(&arr, &5001), None);
        
        assert_eq!(search_insert(&arr, &5000), Ok(2500));
        assert_eq!(search_insert(&arr, &5001), Err(2501));
    }

    #[test]
    fn test_boundary_conditions() {
        let arr = vec![i32::MIN, -1, 0, 1, i32::MAX];
        assert_eq!(search(&arr, &i32::MIN), Some(0));
        assert_eq!(search(&arr, &i32::MAX), Some(4));
        assert_eq!(search(&arr, &0), Some(2));
    }
}