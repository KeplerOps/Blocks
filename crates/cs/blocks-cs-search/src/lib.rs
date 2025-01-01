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

# Usage Example

```rust
use blocks_cs_search::algorithms::linear_search;

let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
assert_eq!(linear_search(&numbers, &4), Some(2));
assert_eq!(linear_search(&numbers, &7), None);
```

# Features
- Generic implementations that work with any type implementing required traits
- Comprehensive test suites including edge cases and performance tests
- Detailed documentation with complexity analysis and usage examples
*/

pub mod algorithms;
