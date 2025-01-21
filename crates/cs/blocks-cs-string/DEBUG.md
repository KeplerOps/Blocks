# Suffix Tree Implementation Debug Log

## Design Plan (Round 0)
1. Implemented Ukkonen's algorithm with:
   - Online construction in O(n) time
   - Active point tracking (node, length, edge)
   - Suffix links for O(n) complexity
   - Rule 3 optimization (skip/count trick)

2. Key components:
   - Node structure with start/end indices and children map
   - ActivePoint structure for Ukkonen's algorithm
   - SuffixTree struct managing construction and queries
   - Pattern matching using tree traversal

3. Test coverage matches existing implementations:
   - Basic construction
   - Pattern finding (find_all)
   - Empty pattern handling
   - Unicode support
   - Overlapping patterns
   - Long text
   - Case sensitivity

## Test Results (Round 1)
Five failing tests:

1. Position calculation overflow in multiple tests:
```
test_find_all: attempt to subtract with overflow
test_case_sensitivity: attempt to subtract with overflow
test_overlapping_patterns: attempt to subtract with overflow
test_long_text: attempt to subtract with overflow
```
- Issue: In collect_positions(), line 262: `result.push(node_ref.start - pattern_len + 1)`
- Hypothesis: Position calculation is incorrect for leaf nodes. We're subtracting pattern_len from start index without bounds checking.

2. Unicode handling issue:
```
test_unicode_text: index out of bounds: the len is 2 but the index is 2
```
- Issue: In line 190 during pattern matching
- Hypothesis: Not properly handling multi-byte unicode characters in text indexing

## Test Results (Round 2)
After adding bounds checking, still have issues:

1. Overflow still occurring at line 263:
```
test_case_sensitivity: attempt to subtract with overflow
test_find_all: attempt to subtract with overflow
test_long_text: attempt to subtract with overflow
```
- Hypothesis: The bounds check condition is incorrect. Need to verify pattern_len calculation.

2. Unicode issue persists:
```
test_unicode_text: index out of bounds: the len is 2 but the index is 2
```
- Hypothesis: Need to handle unicode character boundaries in pattern matching.

3. New issue with overlapping patterns:
```
test_overlapping_patterns: assertion failed: [] vs [0,1,2,3]
```
- Hypothesis: The bounds check is too restrictive, filtering out valid matches.

## Test Results (Round 3)
After fixing type annotations and pattern length calculation, still have failing tests:

1. The tests are failing but not with overflow errors anymore. This suggests our position calculation logic is now incorrect.
2. Looking at the test failures:
   - test_find_all: Basic pattern matching not working
   - test_case_sensitivity: Case-sensitive matching failing
   - test_overlapping_patterns: Not finding overlapping matches
   - test_unicode_text: Unicode character handling still incorrect
   - test_long_text: Long text pattern matching failing

Hypothesis:
1. The issue is in our character position calculation. We're converting indices incorrectly between byte positions and character positions.
2. We need to track both byte and character positions throughout the tree construction and searching.

Next Steps:
1. Modify the Node structure to track both byte and character positions
2. Update the tree construction to maintain both position types
3. Revise position calculation in find_all() to use correct position type
