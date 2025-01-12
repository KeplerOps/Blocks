/// Binary search implementation for searching sorted slices.
///
/// # Algorithm Overview
/// Binary search is a divide-and-conquer algorithm that finds the position of a target value within a sorted array.
/// The algorithm:
/// 1. Compares the target value to the middle element of the array.
/// 2. If the target value is equal to the middle element, the search is complete.
/// 3. If the target value is less than the middle element, the search continues in the left half of the array.
/// 4. If the target value is greater than the middle element, the search continues in the right half of the array.
///
/// # Performance Characteristics
/// - Time: O(log n) for all cases
/// - Space: O(1)
///
/// # Stability
/// - Not applicable: binary search does not modify the array
///
/// # Examples
/// ```
/// use blocks_cs_search::binary_search::binary_search;
/// let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
/// assert_eq!(binary_search(&arr, 5), Some(4));
/// assert_eq!(binary_search(&arr, 11), None);
/// ```

pub fn binary_search<T: Ord>(arr: &[T], target: T) -> Option<usize> {
    let mut low = 0;
    let mut high = arr.len();

    while low < high {
        let mid = (low + high) / 2;
        if arr[mid] == target {
            return Some(mid);
        } else if arr[mid] < target {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_array() {
        let arr: Vec<i32> = vec![];
        assert_eq!(binary_search(&arr, 1), None);
    }

    #[test]
    fn test_single_element() {
        let arr = vec![1];
        assert_eq!(binary_search(&arr, 1), Some(0));
        assert_eq!(binary_search(&arr, 2), None);
    }

    #[test]
    fn test_sorted_array() {
        let arr = vec![1, 2, 3, 4, 5];
        assert_eq!(binary_search(&arr, 1), Some(0));
        assert_eq!(binary_search(&arr, 3), Some(2));
        assert_eq!(binary_search(&arr, 5), Some(4));
        assert_eq!(binary_search(&arr, 6), None);
    }

    #[test]
    fn test_reverse_sorted_array() {
        let arr = vec![5, 4, 3, 2, 1];
        assert_eq!(binary_search(&arr, 1), None);
        assert_eq!(binary_search(&arr, 3), None);
        assert_eq!(binary_search(&arr, 5), None);
    }

    #[test]
    fn test_all_equal_elements() {
        let arr = vec![1, 1, 1, 1, 1];
        assert_eq!(binary_search(&arr, 1), Some(0));
        assert_eq!(binary_search(&arr, 2), None);
    }

    #[test]
    fn test_two_unique_elements_repeated() {
        let arr = vec![1, 2, 1, 2, 1, 2];
        assert_eq!(binary_search(&arr, 1), None);
        assert_eq!(binary_search(&arr, 2), None);
    }

    #[test]
    fn test_large_random_array() {
        let arr: Vec<i32> = (0..1000).collect();
        assert_eq!(binary_search(&arr, 500), Some(500));
        assert_eq!(binary_search(&arr, 1000), None);
    }

    #[test]
    fn test_large_mostly_sorted_array() {
        let mut arr: Vec<i32> = (0..1000).collect();
        for i in 0..50 {
            arr.swap(i * 2, i * 2 + 1);
        }
        assert_eq!(binary_search(&arr, 500), None);
    }

    #[test]
    fn test_custom_type() {
        #[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
        struct Person {
            name: String,
            age: u32,
        }

        let people = vec![
            Person { name: "Alice".to_string(), age: 30 },
            Person { name: "Bob".to_string(), age: 25 },
            Person { name: "Charlie".to_string(), age: 35 },
            Person { name: "David".to_string(), age: 25 },
        ];

        assert_eq!(binary_search(&people, Person { name: "Bob".to_string(), age: 25 }), Some(1));
        assert_eq!(binary_search(&people, Person { name: "Eve".to_string(), age: 40 }), None);
    }
}
