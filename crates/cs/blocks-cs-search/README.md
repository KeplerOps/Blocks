# Blocks CS Search

This crate provides implementations of various search algorithms in Rust.

## Algorithms

### Array/List Search Algorithms
1. Linear Search ✅
   - Simple and straightforward search algorithm
   - Time Complexity: O(n) for small datasets, O(n/t) for large datasets with parallel search
   - Status: Implemented with parallel optimization

2. Binary Search  
   - Efficient search algorithm for sorted arrays
   - Time Complexity: O(log n)

3. Ternary Search  
   - Divides array into three parts
   - Time Complexity: O(log₃ n)

4. Interpolation Search  
   - Improved variant of binary search for uniformly distributed data
   - Time Complexity: O(log log n) average case, O(n) worst case

5. Jump Search  
   - Block-jumping search algorithm
   - Time Complexity: O(√n)

6. Exponential Search  
   - Also known as doubling or galloping search
   - Time Complexity: O(log n)

7. Fibonacci Search  
   - Uses Fibonacci numbers to divide search space
   - Time Complexity: O(log n)

8. Sublist Search  
   - Search for a sublist within a list
   - Time Complexity: O(m×n) where m and n are lengths of lists

### Graph/Tree Search Algorithms
9. Depth-First Search (DFS)  
   - Explores as far as possible along each branch
   - Time Complexity: O(V + E) where V is vertices and E is edges

10. Breadth-First Search (BFS)  
    - Explores all vertices at present depth before moving to next level
    - Time Complexity: O(V + E) where V is vertices and E is edges

## Features
- Generic type support where applicable
- Comprehensive error handling
- Extensive unit tests
- Performance benchmarks
- Documentation with examples