/*!
This crate provides a collection of string algorithms implemented in Rust.

Each algorithm is implemented with a focus on:
- Performance optimizations
- Memory efficiency 
- Comprehensive testing
- Clear documentation
- Modern Rust idioms

# Available Algorithms

## Pattern Matching
- [`KMP`](mod@algorithms::kmp): Knuth-Morris-Pratt algorithm for efficient string searching
  - O(n + m) time complexity
  - O(m) space for pattern preprocessing
  - No backtracking in main search phase

## Suffix Structures
- [`SuffixArray`](mod@algorithms::suffix_array): Efficient string indexing and searching
  - O(n log n) construction
  - O(m log n) search time
  - O(n) space complexity

# Usage Example

```rust
use blocks_cs_string::algorithms::kmp;

let text = "hello world";
let pattern = "world";
let position = kmp::find_first(text, pattern).expect("Search should succeed");
assert_eq!(position, Some(6));
```

# Features
- Generic implementations that work with any byte sequence
- Comprehensive test suites including edge cases
- Detailed documentation with complexity analysis and examples
*/

pub mod algorithms;
pub mod error;

pub use error::{Result, StringError};
