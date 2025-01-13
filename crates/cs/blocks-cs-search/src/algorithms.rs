/*!
This module provides a collection of search algorithms with different performance characteristics
and trade-offs. Each algorithm is implemented with a focus on performance, safety, and usability.

# Available Algorithms

## BinarySearch
A divide-and-conquer algorithm that offers excellent average-case performance for searching sorted arrays.
- Average case: O(log n)
- Worst case: O(log n)
- Space: O(1)
- Not stable
- In-place search

# Examples
```rust
use blocks_cs_search::algorithms::binary_search;

// Using BinarySearch
let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
assert_eq!(binary_search(&arr, 5), Some(4));
assert_eq!(binary_search(&arr, 11), None);
```
*/

pub mod binary_search;

/// Re-export of [`binary_search::binary_search`].
/// 
/// Provides an efficient, in-place binary search implementation with O(log n) complexity.
/// This implementation handles edge cases and returns the index of the target value if found.
pub use self::binary_search::binary_search;
