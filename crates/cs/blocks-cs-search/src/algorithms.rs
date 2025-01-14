/*!
This module provides a collection of search algorithms with different performance characteristics
and trade-offs. Each algorithm is implemented with a focus on performance, safety, and usability.

# Available Algorithms

## Linear Search
A simple sequential search algorithm that checks each element one by one.
- Time: O(n)
- Space: O(1)
- Works on unsorted data
- No prerequisites on data type beyond equality comparison

## Binary Search
An efficient search algorithm for sorted arrays.
- Time: O(log n)
- Space: O(1)
- Requires sorted data
- Requires type to implement Ord trait

More algorithms coming soon...

# Examples
```rust
use blocks_cs_search::algorithms::{linear, binary};

// Using Linear Search
let numbers = vec![3, 1, 4, 1, 5, 9];
let target = 4;
if let Ok(Some(index)) = linear::search(&numbers, &target) {
    println!("Found {} at index {}", target, index);
}

// Using Binary Search (on sorted data)
let sorted = vec![1, 2, 3, 4, 5, 6];
let target = 4;
if let Ok(Some(index)) = binary::search(&sorted, &target) {
    println!("Found {} at index {}", target, index);
}
```
*/

pub mod linear;
pub mod binary;
pub mod ternary;
pub mod interpolation;
pub mod jump;

/// Re-export of [`linear::search`].
/// 
/// Provides a simple linear search implementation with O(n) complexity.
/// This implementation works with any type that implements PartialEq.
pub use self::linear::search as linear_search;

/// Re-export of [`binary::search`].
/// 
/// Provides an efficient binary search implementation with O(log n) complexity.
/// This implementation requires sorted input and works with any type that implements Ord.
pub use self::binary::search as binary_search;

/// Re-export of [`ternary::search`].
/// 
/// Provides a ternary search implementation with O(log₃ n) complexity.
/// This implementation requires sorted input and works with any type that implements Ord.
pub use self::ternary::search as ternary_search;

/// Re-export of [`interpolation::search`].
/// 
/// Provides an interpolation search implementation with O(log log n) average case complexity
/// for uniformly distributed data. This implementation requires sorted input and works with
/// any type that implements Ord + ToPrimitive.
pub use self::interpolation::search as interpolation_search;

/// Re-export of [`jump::search`].
/// 
/// Provides a jump search implementation with O(√n) complexity.
/// This implementation requires sorted input and works with any type that implements Ord.
pub use self::jump::search as jump_search;