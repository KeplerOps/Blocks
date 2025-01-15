/*!
This module provides a collection of string algorithms for pattern matching and string processing.
Each algorithm is implemented with a focus on performance, safety, and usability.

# Available Algorithms

## Knuth-Morris-Pratt (KMP)
An efficient string searching algorithm that utilizes pattern preprocessing.
- Time: O(n + m) where n is text length and m is pattern length
- Space: O(m) for pattern preprocessing
- Suitable for: Single pattern search in a text
- Features: No backtracking in main search phase

## Rabin-Karp
A string searching algorithm that uses hashing to find exact pattern matches.
- Time: O(n + m) average case, O(nm) worst case
- Space: O(1)
- Suitable for: Multiple pattern search
- Features: Rolling hash function for efficient sliding window

## Boyer-Moore
A highly efficient string searching algorithm that scans characters from right to left.
- Time: O(n/m) best case, O(nm) worst case
- Space: O(k) where k is alphabet size
- Suitable for: Long patterns and large alphabets
- Features: Bad character and good suffix rules for efficient skipping

## Z-Algorithm
A linear time pattern matching algorithm using Z-array preprocessing.
- Time: O(n + m) for all cases
- Space: O(n + m) for concatenated string
- Suitable for: Pattern matching and string properties
- Features: Z-box optimization for efficient matching

# Examples
```rust
use blocks_cs_string::algorithms::kmp;

let text = "hello world";
let pattern = "world";
let positions = kmp::find_all(text, pattern).expect("Search should succeed");
assert_eq!(positions, vec![6]);
```
*/

pub mod kmp;
pub mod rabin_karp;
pub mod boyer_moore;
pub mod z_algorithm;

/// Re-export of [`kmp::find_all`].
/// 
/// Provides an efficient implementation of the Knuth-Morris-Pratt string searching algorithm.
/// Returns all occurrences of a pattern in the given text.
pub use self::kmp::find_all as kmp_find_all;

/// Re-export of [`kmp::find_first`].
/// 
/// Similar to find_all but returns only the first occurrence of the pattern.
pub use self::kmp::find_first as kmp_find_first;

/// Re-export of [`rabin_karp::find_all`].
/// 
/// Provides an efficient implementation of the Rabin-Karp string searching algorithm.
/// Returns all occurrences of a pattern in the given text.
pub use self::rabin_karp::find_all as rabin_karp_find_all;

/// Re-export of [`rabin_karp::find_first`].
/// 
/// Similar to find_all but returns only the first occurrence of the pattern.
pub use self::rabin_karp::find_first as rabin_karp_find_first;

/// Re-export of [`boyer_moore::find_all`].
/// 
/// Provides an efficient implementation of the Boyer-Moore string searching algorithm.
/// Returns all occurrences of a pattern in the given text.
pub use self::boyer_moore::find_all as boyer_moore_find_all;

/// Re-export of [`boyer_moore::find_first`].
/// 
/// Similar to find_all but returns only the first occurrence of the pattern.
pub use self::boyer_moore::find_first as boyer_moore_find_first;

/// Re-export of [`z_algorithm::find_all`].
/// 
/// Provides an efficient implementation of the Z-Algorithm for string searching.
/// Returns all occurrences of a pattern in the given text.
pub use self::z_algorithm::find_all as z_algorithm_find_all;

/// Re-export of [`z_algorithm::find_first`].
/// 
/// Similar to find_all but returns only the first occurrence of the pattern.
pub use self::z_algorithm::find_first as z_algorithm_find_first;
