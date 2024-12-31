/*!
This crate provides a collection of sorting algorithms implemented in Rust.

Each algorithm is implemented with a focus on:
- Performance optimizations
- Memory efficiency
- Comprehensive testing
- Clear documentation
- Modern Rust idioms

# Available Algorithms

## Comparison Sorts
- [QuickSort](algorithms/quicksort/index.html): An efficient, in-place sorting algorithm with O(n log n) average case complexity

# Usage Example

```rust
use blocks_cs_sort::algorithms::quicksort;

let mut numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
quicksort(&mut numbers);
assert_eq!(numbers, vec![1, 1, 2, 3, 3, 4, 5, 5, 6, 9]);
```

# Features
- Generic implementations that work with any type implementing required traits
- Comprehensive test suites including edge cases and performance tests
- Detailed documentation with complexity analysis and usage examples
*/

pub mod algorithms;
