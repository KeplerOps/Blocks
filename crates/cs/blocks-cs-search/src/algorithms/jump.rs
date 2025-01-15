use crate::error::{Result, SearchError};

/// Performs a jump search on a sorted slice to find a target value.
/// Uses block jumping to reduce the number of comparisons needed.
///
/// # Arguments
/// * `data` - A sorted slice of elements to search through
/// * `target` - The value to search for
///
/// # Returns
/// * `Ok(Some(index))` - The index where the target value was found
/// * `Ok(None)` - The target value was not found
/// * `Err(SearchError)` - An error occurred during the search (e.g., unsorted input)
///
/// # Examples
/// ```
/// use blocks_cs_search::algorithms::jump;
///
/// let numbers = vec![1, 2, 3, 4, 5, 6];
/// assert_eq!(jump::search(&numbers, &4), Ok(Some(3)));
/// assert_eq!(jump::search(&numbers, &7), Ok(None));
/// ```
///
/// # Performance
/// * Time: O(√n)
/// * Space: O(1)
///
/// # Type Requirements
/// * `T: Ord` - The type must support total ordering
pub fn search<T: Ord>(data: &[T], target: &T) -> Result<Option<usize>> {
    if data.is_empty() {
        return Ok(None);
    }

    // Verify the slice is sorted
    if !is_sorted(data) {
        return Err(SearchError::InvalidInput(
            "Jump search requires sorted input".to_string(),
        ));
    }

    // Calculate optimal jump size: √n
    let jump_size = (data.len() as f64).sqrt().floor() as usize;
    if jump_size == 0 {
        // For very small arrays, do a simple comparison
        return if &data[0] == target {
            Ok(Some(0))
        } else {
            Ok(None)
        };
    }

    // Find the block where element may be present
    let mut prev = 0;
    let mut step = jump_size;

    // Finding the block
    while step < data.len() && &data[step] <= target {
        prev = step;
        step += jump_size;
        if prev >= data.len() {
            return Ok(None);
        }
    }

    // Adjust step to not exceed array bounds
    step = step.min(data.len());

    // Linear search in the identified block
    for i in prev..step {
        if &data[i] == target {
            return Ok(Some(i));
        }
        if &data[i] > target {
            break;
        }
    }

    Ok(None)
}

/// Checks if a slice is sorted in ascending order
fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
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
    fn test_with_duplicates() {
        let data = vec![1, 2, 2, 2, 3, 4];
        // Should find any occurrence of the duplicate value
        let result = search(&data, &2).unwrap().unwrap();
        assert!(result >= 1 && result <= 3);
    }

    #[test]
    fn test_unsorted_input() {
        let data = vec![3, 1, 4, 1, 5];
        assert!(matches!(
            search(&data, &4),
            Err(SearchError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_large_sorted_dataset() {
        let data: Vec<i32> = (0..10_000).collect();
        assert_eq!(search(&data, &5000), Ok(Some(5000)));
        assert_eq!(search(&data, &10_000), Ok(None));
    }

    #[test]
    fn test_with_strings() {
        let data = vec!["apple", "banana", "orange", "pear"];
        assert_eq!(search(&data, &"orange"), Ok(Some(2)));
        assert_eq!(search(&data, &"grape"), Ok(None));
    }

    #[test]
    fn test_boundary_values() {
        let data = vec![i32::MIN, -5, 0, 5, i32::MAX];
        assert_eq!(search(&data, &i32::MIN), Ok(Some(0)));
        assert_eq!(search(&data, &i32::MAX), Ok(Some(4)));
        assert_eq!(search(&data, &0), Ok(Some(2)));
    }

    #[test]
    fn test_various_jump_sizes() {
        // Test with array sizes that produce different jump sizes
        let data1: Vec<i32> = (0..4).collect();   // jump_size = 2
        let data2: Vec<i32> = (0..9).collect();   // jump_size = 3
        let data3: Vec<i32> = (0..16).collect();  // jump_size = 4

        assert_eq!(search(&data1, &2), Ok(Some(2)));
        assert_eq!(search(&data2, &5), Ok(Some(5)));
        assert_eq!(search(&data3, &10), Ok(Some(10)));
    }

    #[test]
    fn test_jump_past_end() {
        let data = vec![1, 3, 5, 7, 9, 11, 13, 15, 17];
        assert_eq!(search(&data, &20), Ok(None));
    }

    #[test]
    fn test_step_adjustment() {
        let data = vec![1, 3, 5, 7, 9, 11];
        assert_eq!(search(&data, &10), Ok(None));
    }

    #[test]
    fn test_break_on_greater() {
        let data = vec![1, 3, 5, 7, 9, 11];
        assert_eq!(search(&data, &4), Ok(None));
    }

    #[test]
    fn test_prev_exceeds_len() {
        let data = vec![1, 3, 5, 7, 9, 11, 13, 15, 17];
        let target = 100;  // This will cause prev to exceed data.len()
        assert_eq!(search(&data, &target), Ok(None));
    }

    #[test]
    fn test_step_adjustment_with_target() {
        let data = vec![1, 3, 5, 7, 9, 11, 13, 15];
        assert_eq!(search(&data, &14), Ok(None));
    }

    #[test]
    fn test_break_in_linear_search() {
        let data = vec![1, 3, 5, 7, 9, 11, 13, 15];
        assert_eq!(search(&data, &6), Ok(None));
    }
}