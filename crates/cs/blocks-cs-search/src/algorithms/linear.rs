/// Performs a linear search on a slice to find a target element.
///
/// # Algorithm Details
/// - Time Complexity: O(n) where n is the length of the array
/// - Space Complexity: O(1)
/// - In-place: Yes
/// - Stable: Yes (maintains relative order of equal elements)
///
/// # Implementation Notes
/// - Iterates through the array sequentially
/// - Returns the index of the first occurrence of the target element
/// - Returns None if the element is not found
///
/// # Examples
/// ```
/// use blocks_cs_search::algorithms::linear::search;
/// let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
/// assert_eq!(search(&numbers, &4), Some(2));
/// assert_eq!(search(&numbers, &7), None);
/// ```
pub fn search<T: PartialEq>(arr: &[T], target: &T) -> Option<usize> {
    for (index, item) in arr.iter().enumerate() {
        if item == target {
            return Some(index);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;

    // Basic functionality tests
    #[test]
    fn test_empty_array() {
        let arr: Vec<i32> = vec![];
        assert_eq!(search(&arr, &1), None);
    }

    #[test]
    fn test_single_element_found() {
        let arr = vec![1];
        assert_eq!(search(&arr, &1), Some(0));
    }

    #[test]
    fn test_single_element_not_found() {
        let arr = vec![1];
        assert_eq!(search(&arr, &2), None);
    }

    #[test]
    fn test_multiple_elements_found() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(search(&arr, &5), Some(2));
    }

    #[test]
    fn test_multiple_elements_not_found() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(search(&arr, &4), None);
    }

    // Edge cases
    #[test]
    fn test_first_element() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(search(&arr, &1), Some(0));
    }

    #[test]
    fn test_last_element() {
        let arr = vec![1, 3, 5, 7, 9];
        assert_eq!(search(&arr, &9), Some(4));
    }

    #[test]
    fn test_duplicates() {
        let arr = vec![1, 3, 5, 5, 7, 9];
        assert_eq!(search(&arr, &5), Some(2)); // Returns first occurrence
    }

    // Type tests
    #[test]
    fn test_string_slice() {
        let arr = vec!["apple", "banana", "orange"];
        assert_eq!(search(&arr, &"banana"), Some(1));
        assert_eq!(search(&arr, &"grape"), None);
    }

    #[test]
    fn test_string() {
        let arr = vec![
            String::from("apple"),
            String::from("banana"),
            String::from("orange"),
        ];
        assert_eq!(search(&arr, &String::from("banana")), Some(1));
        assert_eq!(search(&arr, &String::from("grape")), None);
    }

    // Custom type tests
    #[derive(Debug, Eq, PartialEq)]
    struct Person {
        name: String,
        age: u32,
    }

    #[test]
    fn test_custom_type() {
        let people = vec![
            Person { name: "Alice".to_string(), age: 30 },
            Person { name: "Bob".to_string(), age: 25 },
            Person { name: "Charlie".to_string(), age: 35 },
        ];

        let target = Person { name: "Bob".to_string(), age: 25 };
        assert_eq!(search(&people, &target), Some(1));

        let not_found = Person { name: "David".to_string(), age: 40 };
        assert_eq!(search(&people, &not_found), None);
    }

    // Large array tests
    #[test]
    fn test_large_array() {
        let arr: Vec<i32> = (0..10000).collect();
        assert_eq!(search(&arr, &9999), Some(9999)); // Last element
        assert_eq!(search(&arr, &10000), None); // Not found
    }

    #[test]
    fn test_large_array_with_duplicates() {
        let arr: Vec<i32> = (0..1000).map(|x| x % 100).collect(); // Numbers 0-99 repeated 10 times
        assert_eq!(search(&arr, &50), Some(50)); // First occurrence
    }
}