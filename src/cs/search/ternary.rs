use crate::error::{Result, SearchError};

/// Performs a ternary search on a sorted slice to find a target value.
/// Divides the search interval into three parts and determines which part to search next.
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
/// use blocks_cs_search::algorithms::ternary;
///
/// let numbers = vec![1, 2, 3, 4, 5, 6];
/// assert_eq!(ternary::search(&numbers, &4), Ok(Some(3)));
/// assert_eq!(ternary::search(&numbers, &7), Ok(None));
/// ```
///
/// # Performance
/// * Time: O(log₃ n) - Base-3 logarithmic time
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
            "Ternary search requires sorted input".to_string(),
        ));
    }

    let mut left = 0;
    let mut right = data.len() - 1;

    while left <= right {
        // Early return if array is too small
        if right - left < 2 {
            if &data[left] == target {
                return Ok(Some(left));
            }
            if right > left && &data[right] == target {
                return Ok(Some(right));
            }
            return Ok(None);
        }

        // Calculate the two mid points that divide the range into three parts
        let mid1 = left + (right - left) / 3;
        let mid2 = right - (right - left) / 3;

        // Check if target is at either mid point
        if &data[mid1] == target {
            return Ok(Some(mid1));
        }
        if &data[mid2] == target {
            return Ok(Some(mid2));
        }

        // Determine which third to search next
        if target < &data[mid1] {
            right = mid1 - 1;
        } else if target > &data[mid2] {
            left = mid2 + 1;
        } else {
            left = mid1 + 1;
            right = mid2 - 1;
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
    fn test_two_elements_found_first() {
        let data = vec![1, 2];
        assert_eq!(search(&data, &1), Ok(Some(0)));
    }

    #[test]
    fn test_two_elements_found_second() {
        let data = vec![1, 2];
        assert_eq!(search(&data, &2), Ok(Some(1)));
    }

    #[test]
    fn test_multiple_elements_found_first() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(search(&data, &1), Ok(Some(0)));
    }

    #[test]
    fn test_multiple_elements_found_last() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(search(&data, &9), Ok(Some(8)));
    }

    #[test]
    fn test_multiple_elements_found_middle() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(search(&data, &5), Ok(Some(4)));
    }

    #[test]
    fn test_multiple_elements_not_found() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(search(&data, &10), Ok(None));
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
}