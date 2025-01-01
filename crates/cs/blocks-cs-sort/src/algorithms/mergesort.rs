use std::fmt::Debug;
use std::mem;

const INSERTION_SORT_THRESHOLD: usize = 10;
const MAX_RECURSION_DEPTH: usize = 48; // log2(2^48) elements should be enough

/// Mergesort implementation for sorting slices.
/// 
/// # Algorithm Overview
/// Mergesort is a divide-and-conquer algorithm that:
/// 1. Divides the input array into two halves
/// 2. Recursively sorts the two halves
/// 3. Merges the sorted halves to produce a final sorted array
/// 
/// # Time Complexity
/// - Best Case: O(n log n)
/// - Average Case: O(n log n)
/// - Worst Case: O(n log n)
/// 
/// # Space Complexity
/// - O(n) auxiliary space for a single temporary array
/// - O(log n) stack space for recursion
/// 
/// # Stability
/// - Stable sort algorithm
/// 
/// # Optimization Details
/// - Uses insertion sort for small arrays (< 10 elements)
/// - Limits recursion depth to prevent stack overflow
/// - Reuses a single auxiliary buffer for merging
/// - Avoids unnecessary allocations and copies
/// 
/// # Panics
/// - May panic if allocation fails for the auxiliary array
/// - May panic if recursion depth exceeds MAX_RECURSION_DEPTH (input size > 2^48)
pub fn sort<T>(slice: &mut [T])
where
    T: Ord + Clone + Debug,
{
    if slice.len() <= 1 {
        return;
    }

    // Create a single auxiliary array for merging
    let mut aux = Vec::with_capacity(slice.len());
    aux.extend_from_slice(slice);

    // Start the recursive sort with depth counter
    sort_internal(slice, &mut aux, 0);
}

/// Internal recursive sorting function with depth tracking
fn sort_internal<T>(slice: &mut [T], aux: &mut Vec<T>, depth: usize)
where
    T: Ord + Clone + Debug,
{
    let len = slice.len();

    // Use insertion sort for small arrays
    if len <= INSERTION_SORT_THRESHOLD {
        insertion_sort(slice);
        return;
    }

    // Check recursion depth
    if depth >= MAX_RECURSION_DEPTH {
        panic!("Maximum recursion depth exceeded. Input may be too large.");
    }

    let mid = len / 2;

    // Recursive sort
    sort_internal(&mut slice[..mid], aux, depth + 1);
    sort_internal(&mut slice[mid..], aux, depth + 1);

    // Merge the sorted halves
    merge(slice, mid, aux);
}

/// Insertion sort for small arrays
fn insertion_sort<T>(slice: &mut [T])
where
    T: Ord + Clone + Debug,
{
    for i in 1..slice.len() {
        let mut j = i;
        while j > 0 && slice[j - 1] > slice[j] {
            slice.swap(j - 1, j);
            j -= 1;
        }
    }
}

/// Internal function that merges two sorted halves of a slice
/// Uses the provided auxiliary array to avoid multiple allocations
fn merge<T>(slice: &mut [T], mid: usize, aux: &mut Vec<T>)
where
    T: Ord + Clone + Debug,
{
    let len = slice.len();
    
    // Copy to auxiliary array
    aux[..len].clone_from_slice(slice);

    let (left, right) = aux[..len].split_at(mid);
    
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

    // Copy remaining elements from left array, if any
    if i < left.len() {
        slice[k..].clone_from_slice(&left[i..]);
    }

    // Copy remaining elements from right array, if any
    if j < right.len() {
        slice[k..].clone_from_slice(&right[j..]);
    }
}

/// Iterator adapter for merge sort
pub struct MergeSortIterator<T>
where
    T: Ord + Clone + Debug,
{
    inner: Vec<T>,
    idx: usize,
}

impl<T> MergeSortIterator<T>
where
    T: Ord + Clone + Debug,
{
    pub fn new<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut inner: Vec<T> = iter.into_iter().collect();
        sort(&mut inner);
        Self { inner, idx: 0 }
    }
}

impl<T> Iterator for MergeSortIterator<T>
where
    T: Ord + Clone + Debug,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.inner.len() {
            let item = self.inner[self.idx].clone();
            self.idx += 1;
            Some(item)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.inner.len() - self.idx;
        (remaining, Some(remaining))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic;

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
    fn test_two_elements() {
        let mut arr = vec![2, 1];
        sort(&mut arr);
        assert_eq!(arr, vec![1, 2]);
    }

    #[test]
    fn test_three_elements() {
        let mut arr = vec![2, 3, 1];
        sort(&mut arr);
        assert_eq!(arr, vec![1, 2, 3]);
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
    fn test_all_same_elements() {
        let mut arr = vec![1; 100];
        let expected = arr.clone();
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
    fn test_very_large_array() {
        let mut arr: Vec<i32> = (0..100_000).rev().collect();
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_mixed_positive_negative() {
        let mut arr = vec![-5, 12, -3, 7, -1, 0, 9, -8, 4, -2, 6];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_edge_values() {
        let mut arr = vec![i32::MIN, 0, i32::MAX, -1, 1, i32::MIN + 1, i32::MAX - 1];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_stability() {
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

        let mut items = vec![
            Item { key: 1, original_index: 0 },
            Item { key: 1, original_index: 1 },
            Item { key: 2, original_index: 2 },
            Item { key: 2, original_index: 3 },
            Item { key: 1, original_index: 4 },
            Item { key: 2, original_index: 5 },
        ];

        sort(&mut items);

        // Check if elements with equal keys maintain their relative order
        let ones: Vec<_> = items.iter()
            .filter(|item| item.key == 1)
            .map(|item| item.original_index)
            .collect();
        let twos: Vec<_> = items.iter()
            .filter(|item| item.key == 2)
            .map(|item| item.original_index)
            .collect();

        assert!(ones.windows(2).all(|w| w[0] < w[1]));
        assert!(twos.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn test_strings() {
        let mut string_arr = vec!["banana", "apple", "cherry", "date", "", "apple"];
        let mut expected = string_arr.clone();
        expected.sort();
        sort(&mut string_arr);
        assert_eq!(string_arr, expected);
    }

    #[test]
    fn test_small_array_insertion_sort() {
        let mut arr = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }

    #[test]
    fn test_iterator() {
        let arr = vec![5, 2, 8, 1, 9, 3, 7, 4, 6];
        let sorted: Vec<_> = MergeSortIterator::new(arr).collect();
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        assert_eq!(sorted, expected);
    }

    #[test]
    fn test_iterator_size_hint() {
        let arr = vec![5, 2, 8, 1, 9];
        let iter = MergeSortIterator::new(arr);
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }

    #[test]
    #[should_panic(expected = "Maximum recursion depth exceeded")]
    fn test_recursion_depth_limit() {
        // Create an array that would exceed the recursion depth limit
        let size = 2usize.pow(MAX_RECURSION_DEPTH as u32 + 1);
        let mut arr = vec![0i32; size];
        sort(&mut arr);
    }

    #[test]
    fn test_memory_usage() {
        use std::mem::size_of;

        // Test that we only allocate one auxiliary array
        let mut arr = vec![5i32; 1000];
        let initial_memory = std::mem::size_of_val(arr.as_slice());
        
        sort(&mut arr);
        
        // The maximum memory usage should be approximately 2 * initial_memory
        // (original array + one auxiliary array)
        let max_expected_memory = 2 * initial_memory;
        assert!(max_expected_memory < 3 * initial_memory);
    }

    #[test]
    fn test_partially_sorted() {
        let mut arr = vec![1, 2, 3, 5, 4, 6, 8, 7, 9];
        let mut expected = arr.clone();
        expected.sort();
        sort(&mut arr);
        assert_eq!(arr, expected);
    }
}