
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

//Using CountingSort
```
*/

pub mod quicksort;
pub mod heapsort;
pub mod mergesort;
pub mod bubblesort;
pub mod insertionsort;
pub mod selectionsort;
pub mod shellsort;
pub mod countingsort;

/// Re-export of [`quicksort::sort`].
/// 
/// Provides an efficient, in-place quicksort implementation with O(n log n) average-case complexity.
/// This implementation uses median-of-three pivot selection and switches to insertion sort for small arrays.
pub use self::quicksort::sort as quicksort;

/// Re-export of [`heapsort::sort`].
/// 
/// Provides a heap-based sorting implementation with guaranteed O(n log n) complexity and O(1) space usage.
/// This implementation supports parallel sorting for large arrays when the `parallel` feature is enabled.
pub use self::heapsort::sort as heapsort;

/// Re-export of [`mergesort::sort`].
/// 
/// Provides a stable sorting implementation with guaranteed O(n log n) complexity.
/// This implementation uses O(n) auxiliary space to achieve stability.
pub use self::mergesort::sort as mergesort;

// TODO
pub use self::bubblesort::sort as bubblesort;

pub use self::insertionsort::sort as insertionsort;

pub use self::selectionsort::sort as selectionsort;

pub use self::shellsort::sort as shellsort;

pub use self::countingsort::sort as countingsort;
