/*!
This crate provides a collection of search algorithms implemented in Rust.

Each algorithm is implemented with a focus on:
- Performance optimizations
- Memory efficiency
- Comprehensive testing
- Clear documentation
- Modern Rust idioms

# Available Algorithms

## Sequential Search
- [Linear Search](algorithms/linear/index.html): A simple, sequential search algorithm with O(n) time complexity

## Binary Search
- [Binary Search](algorithms/binary/index.html): An efficient search algorithm for sorted data with O(log n) time complexity
- [Binary Search Insert](algorithms/binary/index.html#search_insert): Finds insertion point in sorted data, useful for maintaining sorted collections

# Usage Example

```rust
use blocks_cs_search::algorithms::{binary_search, linear_search};

// Linear search - works on unsorted data
let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
assert_eq!(linear_search(&numbers, &4), Some(2));

// Binary search - requires sorted data
let sorted = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
assert_eq!(binary_search(&sorted, &6), Some(5));
```

# Features
- Generic implementations that work with any type implementing required traits
- Comprehensive test suites including edge cases and performance tests
- Property-based testing to verify correctness
- Detailed documentation with complexity analysis and usage examples
- Benchmarking suite for performance comparison
*/

pub mod algorithms;
