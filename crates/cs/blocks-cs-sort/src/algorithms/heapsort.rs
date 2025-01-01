use std::fmt::Debug;

/// Heapsort implementation for sorting slices.
/// 
/// # Algorithm Overview
/// Heapsort is a comparison-based sorting algorithm that uses a binary heap data structure.
/// The algorithm:
/// 1. Builds a max-heap from the input array
/// 2. Repeatedly extracts the maximum element and places it at the end
/// 3. Maintains the heap property after each extraction
/// 
/// # Time Complexity
/// - Best Case: O(n log n)
/// - Average Case: O(n log n)
/// - Worst Case: O(n log n)
/// 
/// # Space Complexity
/// - O(1) auxiliary space
/// 
/// # Stability
/// - Not stable
pub fn sort<T>(slice: &mut [T])
where
    T: Ord + Clone + Debug,
{
    if slice.len() <= 1 {
        return;
    }

    // Build max heap
    build_max_heap(slice);

    // Extract elements from heap one by one
    for i in (1..slice.len()).rev() {
        slice.swap(0, i);
        heapify(slice, i, 0);
    }
}

/// Builds a max heap from an arbitrary array
fn build_max_heap<T>(slice: &mut [T])
where
    T: Ord + Clone + Debug,
{
    let len = slice.len();
    // Start from last non-leaf node and heapify all nodes
    for i in (0..len/2).rev() {
        heapify(slice, len, i);
    }
}

/// Maintains the max heap property for a subtree rooted at index i
fn heapify<T>(slice: &mut [T], len: usize, root: usize)
where
    T: Ord + Clone + Debug,
{
    let mut largest = root;
    let left = 2 * root + 1;
    let right = 2 * root + 2;

    // Compare with left child
    if left < len && slice[left] > slice[largest] {
        largest = left;
    }

    // Compare with right child
    if right < len && slice[right] > slice[largest] {
        largest = right;
    }

    // If largest is not root
    if largest != root {
        slice.swap(root, largest);
        // Recursively heapify the affected sub-tree
        heapify(slice, len, largest);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_slice() {
        let mut arr: Vec<i32> = vec![];
        sort(&mut arr);
        assert_eq!(arr, vec![]);
    }

    #[test]
    fn test_single_element() {
        let mut arr = vec![1];
        sort(&mut arr);
        assert_eq!(arr, vec![1]);
    }

    #[test]
    fn test_sorted_array() {
        let mut arr = vec![1, 2, 3, 4, 5];
        sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_reverse_sorted() {
        let mut arr = vec![5, 4, 3, 2, 1];
        sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_random_order() {
        let mut arr = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_duplicate_elements() {
        let mut arr = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3, 5];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_large_array() {
        let mut arr: Vec<i32> = (0..1000).rev().collect();
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_different_types() {
        // Test with floating point numbers
        let mut float_arr = vec![3.14, 1.41, 2.71, 0.58];
        let mut expected = float_arr.clone();
        expected.sort_by(|a, b| a.partial_cmp(b).unwrap());
        sort(&mut float_arr);
        assert_eq!(float_arr, expected);

        // Test with strings
        let mut string_arr = vec!["banana", "apple", "cherry", "date"];
        let mut expected = string_arr.clone();
        expected.sort();
        sort(&mut string_arr);
        assert_eq!(string_arr, expected);
    }

    #[test]
    fn test_heap_property() {
        let mut arr = vec![4, 10, 3, 5, 1];
        build_max_heap(&mut arr);
        
        // Test max-heap property: parent should be greater than or equal to its children
        for i in 0..arr.len() {
            let left = 2 * i + 1;
            let right = 2 * i + 2;
            
            if left < arr.len() {
                assert!(arr[i] >= arr[left]);
            }
            if right < arr.len() {
                assert!(arr[i] >= arr[right]);
            }
        }
    }
}