#![warn(missing_docs)]
#![warn(clippy::missing_docs_in_private_items)]

/// Performs a linear search on a slice to find a target element.
///
/// This function implements a simple linear (sequential) search algorithm that finds
/// the first occurrence of a given element in a slice. It's particularly useful when:
/// - The data is not sorted
/// - The dataset is small
/// - Memory constraints prevent sorting or using more complex data structures
/// - The data is rarely searched (making preprocessing costs unjustifiable)
///
/// # Algorithm Details
/// - Time Complexity: O(n) where n is the length of the slice
/// - Space Complexity: O(1)
/// - In-place: Yes
/// - Stable: Yes (maintains relative order of equal elements)
/// - Early exit: Yes (returns as soon as element is found)
///
/// # Type Parameters
/// - `T`: The type of elements in the slice, must implement [`PartialEq`]
///
/// # Arguments
/// * `arr` - A slice of elements to search through
/// * `target` - The element to find in the slice
///
/// # Returns
/// - `Some(index)` if the element is found, where `index` is the position of the first occurrence
/// - `None` if the element is not present in the slice
///
/// # Examples
/// Basic usage with integers:
/// ```
/// use blocks_cs_search::algorithms::linear::search;
/// let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
/// assert_eq!(search(&numbers, &4), Some(2));
/// assert_eq!(search(&numbers, &7), None);
/// ```
///
/// Works with any type that implements [`PartialEq`]:
/// ```
/// use blocks_cs_search::algorithms::linear::search;
/// let words = vec!["apple", "banana", "cherry"];
/// assert_eq!(search(&words, &"banana"), Some(1));
/// ```
///
/// # Performance Considerations
/// - For sorted data, consider using [`binary_search`] instead
/// - For frequently searched datasets, consider using a [`HashSet`] or [`HashMap`]
/// - For very large datasets, consider using more efficient search algorithms or data structures
///
/// [`binary_search`]: https://doc.rust-lang.org/std/primitive.slice.html#method.binary_search
/// [`HashSet`]: https://doc.rust-lang.org/std/collections/struct.HashSet.html
/// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
#[inline]
pub fn search<T: PartialEq>(arr: &[T], target: &T) -> Option<usize> {
    // SIMD would be beneficial here for primitive types, but that requires
    // unsafe code and architecture-specific optimizations. For an enterprise
    // implementation, we might want to add a feature flag for SIMD support.
    arr.iter()
        .position(|item| item == target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    // Property-based tests
    #[quickcheck]
    fn prop_found_element_exists(xs: Vec<i32>, index: usize) -> bool {
        if xs.is_empty() || index >= xs.len() {
            return true;
        }
        let target = xs[index];
        match search(&xs, &target) {
            Some(found_index) => xs[found_index] == target && 
                                // Verify it's the first occurrence
                                !xs[..found_index].contains(&target),
            None => false // If we picked an element from the vector, it must be found
        }
    }

    #[quickcheck]
    fn prop_not_found_element_absent(xs: Vec<i32>, x: i32) -> bool {
        match search(&xs, &x) {
            Some(index) => xs[index] == x,
            None => !xs.contains(&x)
        }
    }

    #[quickcheck]
    fn prop_maintains_invariants(xs: Vec<i32>, x: i32) -> bool {
        let result = search(&xs, &x);
        
        // If found, index must be valid and element must match
        result.map_or(true, |i| i < xs.len() && xs[i] == x) &&
        // If not found, element must not exist in array
        result.is_none() == !xs.contains(&x)
    }

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