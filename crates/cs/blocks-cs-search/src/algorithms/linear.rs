use crate::error::{Result, SearchError};
use rayon::prelude::*;

/// Threshold for switching to parallel search
const PARALLEL_THRESHOLD: usize = 1024;

/// Performs a linear search on a slice to find the first occurrence of a target value.
/// For large datasets (>= 1024 elements), automatically switches to parallel search.
///
/// # Arguments
/// * `data` - A slice of elements to search through
/// * `target` - The value to search for
///
/// # Returns
/// * `Ok(Some(index))` - The index of the first occurrence of the target value
/// * `Ok(None)` - The target value was not found
/// * `Err(SearchError)` - An error occurred during the search
///
/// # Examples
/// ```
/// use blocks_cs_search::algorithms::linear;
///
/// let numbers = vec![3, 1, 4, 1, 5, 9];
/// assert_eq!(linear::search(&numbers, &4), Ok(Some(2)));
/// assert_eq!(linear::search(&numbers, &6), Ok(None));
///
/// // For large datasets, automatically uses parallel search
/// let large_data: Vec<i32> = (0..10_000).collect();
/// assert_eq!(linear::search(&large_data, &5000), Ok(Some(5000)));
/// ```
///
/// # Performance
/// * Small datasets (< 1024 elements):
///   - Time: O(n)
///   - Space: O(1)
/// * Large datasets (>= 1024 elements):
///   - Time: O(n/t) where t is the number of available threads
///   - Space: O(1)
///
/// # Type Requirements
/// * `T: PartialEq + Sync` - The type must support equality comparison and be thread-safe
pub fn search<T: PartialEq + Sync>(data: &[T], target: &T) -> Result<Option<usize>> {
    if data.is_empty() {
        return Ok(None);
    }

    // Use parallel search for large datasets
    if data.len() >= PARALLEL_THRESHOLD {
        return parallel_search(data, target);
    }

    // Sequential search for smaller datasets
    for (index, item) in data.iter().enumerate() {
        if item == target {
            return Ok(Some(index));
        }
    }

    Ok(None)
}

/// Performs a parallel linear search on large datasets
fn parallel_search<T: PartialEq + Sync>(data: &[T], target: &T) -> Result<Option<usize>> {
    data.par_iter()
        .position_first(|item| item == target)
        .map(Ok)
        .unwrap_or(Ok(None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_slice() {
        let data: Vec<i32> = vec![];
        assert_eq!(search(&data, &5), Ok(None));
    }

    #[test]
    fn test_single_element_found() {
        let data = vec![5];
        assert_eq!(search(&data, &5), Ok(Some(0)));
    }

    #[test]
    fn test_single_element_not_found() {
        let data = vec![5];
        assert_eq!(search(&data, &3), Ok(None));
    }

    #[test]
    fn test_multiple_elements_found_first() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(search(&data, &1), Ok(Some(0)));
    }

    #[test]
    fn test_multiple_elements_found_last() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(search(&data, &5), Ok(Some(4)));
    }

    #[test]
    fn test_multiple_elements_found_middle() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(search(&data, &3), Ok(Some(2)));
    }

    #[test]
    fn test_multiple_elements_not_found() {
        let data = vec![1, 2, 3, 4, 5];
        assert_eq!(search(&data, &6), Ok(None));
    }

    #[test]
    fn test_with_duplicates_finds_first() {
        let data = vec![1, 2, 2, 3, 2, 4];
        assert_eq!(search(&data, &2), Ok(Some(1)));
    }

    #[test]
    fn test_with_strings() {
        let data = vec!["apple", "banana", "orange"];
        assert_eq!(search(&data, &"banana"), Ok(Some(1)));
        assert_eq!(search(&data, &"grape"), Ok(None));
    }

    #[test]
    fn test_parallel_search_large_dataset() {
        // Create a dataset larger than PARALLEL_THRESHOLD
        let data: Vec<i32> = (0..PARALLEL_THRESHOLD + 100).collect();
        let target = PARALLEL_THRESHOLD as i32 + 50;
        
        assert_eq!(search(&data, &target), Ok(Some(PARALLEL_THRESHOLD + 50)));
        assert_eq!(search(&data, &(PARALLEL_THRESHOLD as i32 + 200)), Ok(None));
    }

    #[test]
    fn test_parallel_search_with_duplicates() {
        let mut data: Vec<i32> = (0..PARALLEL_THRESHOLD + 100).collect();
        // Add some duplicates
        data[PARALLEL_THRESHOLD + 20] = 5;
        data[PARALLEL_THRESHOLD + 30] = 5;
        
        // Should find the first occurrence
        assert_eq!(search(&data, &5), Ok(Some(5)));
    }
}