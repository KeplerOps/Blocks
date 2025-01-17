# String Algorithms

1. Knuth–Morris–Pratt (KMP)  
2. Rabin–Karp  
3. Boyer–Moore  
4. Z-Algorithm  
5. Aho–Corasick  
6. Suffix Array Construction (prefix doubling)  
7. Suffix Automaton  
8. Suffix Tree Construction (Ukkonen's)  
9. Rolling Hash (for pattern matching)  
10. Manacher's Algorithm (longest palindromic substring)

## Aho-Corasick Algorithm Design

### Overview

The Aho-Corasick algorithm is a string matching algorithm that can efficiently find multiple patterns in a text simultaneously. It constructs a finite state machine from the patterns and then uses it to scan the text in a single pass.

### Data Structures

#### TrieNode

```rust
struct TrieNode {
    // Children nodes indexed by character
    children: HashMap<char, usize>,  // usize is node index
    // Failure link to longest proper suffix
    failure: Option<usize>,
    // Dictionary patterns ending at this node
    patterns: Vec<usize>,  // Indices into patterns array
    // Depth in trie (optimization)
    depth: usize,
}
```

#### Automaton

```rust
struct AhoCorasick {
    // All nodes in the automaton
    nodes: Vec<TrieNode>,
    // Original patterns for reporting matches
    patterns: Vec<String>,
    // Root node is always at index 0
    root: usize,
}
```

### Algorithm Phases

#### 1. Trie Construction
- Create root node
- For each pattern:
  - Start at root
  - For each char:
    - Create/reuse node
    - Update pattern indices
- Time: O(sum of pattern lengths)
- Space: O(sum of pattern lengths)

#### 2. Failure Link Construction
- BFS traversal from root
- For each node:
  - Get parent's failure link
  - Follow failure + character transitions
  - Merge pattern matches
- Time: O(sum of pattern lengths)
- Space: O(1) extra

### 3. Pattern Matching
- Start at root
- For each text character:
  - Follow transitions/failure links
  - Report matches at current node
- Time: O(text length + number of matches)
- Space: O(1) extra

## Implementation Considerations

### Memory Management
- Use indices instead of references to avoid lifetime issues
- Store patterns in main struct for lifetime management
- Consider using SmallVec for small pattern lists

### Performance Optimizations
- Cache failure link transitions
- Use byte-level matching for ASCII
- Batch pattern reporting
- Consider dense vs sparse node storage

### Error Handling
- Validate input patterns (non-empty, valid UTF-8)
- Handle allocation failures gracefully
- Consider using custom error type

### API Design
```rust
impl AhoCorasick {
    // Constructor
    pub fn new(patterns: Vec<String>) -> Self;
    
    // Main search function
    pub fn find_matches<'a>(&'a self, text: &'a str) 
        -> impl Iterator<Item = Match> + 'a;
        
    // Optional configuration
    pub fn builder() -> AhoCorasickBuilder;
}

// Match result
pub struct Match {
    pub pattern_index: usize,
    pub start: usize,
    pub end: usize,
}
```

## Testing Strategy

1. Basic Functionality
   - Single pattern matching
   - Multiple pattern matching
   - Overlapping patterns
   - Empty text/patterns
   
2. Edge Cases
   - Unicode handling
   - Very long patterns
   - Many short patterns
   - Patterns with common prefixes
   
3. Performance
   - Large text corpus
   - Many patterns
   - Worst-case scenarios

## Future Improvements
- SIMD optimization for ASCII
- Compressed node storage
- Streaming interface
- Case insensitive matching
- Memory usage tuning
