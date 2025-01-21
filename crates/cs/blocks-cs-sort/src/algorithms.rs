/*!
This module provides a collection of sorting algorithms with different performance characteristics
and trade-offs. Each algorithm is implemented with a focus on performance, safety, and usability.

# Available Algorithms

## QuickSort
A divide-and-conquer algorithm that offers excellent average-case performance and in-place sorting.
- Average case: O(n log n)
- Worst case: O(n²)
- Space: O(log n)
- Not stable
- In-place sorting

## HeapSort
A comparison-based algorithm using a binary heap data structure, offering consistent performance.
- Time: O(n log n) for all cases
- Space: O(1)
- Not stable
- In-place sorting

## MergeSort
A divide-and-conquer algorithm that guarantees stability and consistent performance.
- Time: O(n log n) for all cases
- Space: O(n)
- Stable
- Not in-place sorting

# Feature Flags
- `parallel`: Enables parallel sorting for large arrays
- `simd`: Enables SIMD optimizations for numeric types

# Examples
```rust
use blocks_cs_sort::algorithms::{quicksort, heapsort};

// Using QuickSort
let mut numbers = vec![3, 1, 4, 1, 5, 9];
quicksort(&mut numbers);
assert_eq!(numbers, vec![1, 1, 3, 4, 5, 9]);

// Using HeapSort
let mut numbers = vec![3, 1, 4, 1, 5, 9];
heapsort(&mut numbers).expect("Sort should succeed");
assert_eq!(numbers, vec![1, 1, 3, 4, 5, 9]);

// Using MergeSort
let mut numbers = vec![3, 1, 4, 1, 5, 9];
mergesort(&mut numbers);
assert_eq!(numbers, vec![1, 1, 3, 4, 5, 9]);

// Using BubbleSort

// Using InsertionSort

// Using SelectionSort

// Using ShellSort

// Using CountingSort

// Using RadixSort

// Using BucketSort
```
*/

/// Sorting algorithms module.
///
/// This module provides various sorting algorithms with different performance characteristics.
/// Each algorithm is implemented as a separate module with its own tests and documentation.
///
/// # Examples
///
/// ```
/// use blocks_cs_sort::algorithms::mergesort;
///
/// let mut numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
/// mergesort::sort(&mut numbers).expect("Sort should succeed");
/// assert_eq!(numbers, vec![1, 1, 2, 3, 3, 4, 5, 5, 6, 9]);
/// ```
pub mod bubblesort;
pub mod bucketsort;
pub mod countingsort;
pub mod heapsort;
pub mod insertionsort;
pub mod mergesort;
pub mod quicksort;
pub mod radixsort;
pub mod selectionsort;
pub mod shellsort;

// Re-export sort functions for convenience
pub use self::bubblesort::sort as bubblesort;
pub use self::bucketsort::sort as bucketsort;
pub use self::countingsort::sort as countingsort;
pub use self::heapsort::sort as heapsort;
pub use self::insertionsort::sort as insertionsort;
pub use self::mergesort::sort as mergesort;
pub use self::quicksort::sort as quicksort;
pub use self::radixsort::sort as radixsort;
pub use self::selectionsort::sort as selectionsort;
pub use self::shellsort::sort as shellsort;
