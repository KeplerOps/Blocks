/*!
This module provides a collection of graph algorithms with different use cases and performance
characteristics. Each algorithm is implemented with a focus on performance, safety, and usability.

# Available Algorithms

## Shortest Path Algorithms

### Dijkstra's Algorithm
Single-source shortest path algorithm for graphs with non-negative weights.
- Time: O((V + E) log V) with binary heap
- Space: O(V)
- Requires: Non-negative weights

### Bellman-Ford Algorithm
Single-source shortest path algorithm that can handle negative weights.
- Time: O(VE)
- Space: O(V)
- Can detect negative cycles

### Floyd-Warshall Algorithm
All-pairs shortest path algorithm.
- Time: O(V³)
- Space: O(V²)
- Works with negative weights

### Johnson's Algorithm
All-pairs shortest path algorithm optimized for sparse graphs.
- Time: O(VE log V)
- Space: O(V²)
- Better than Floyd-Warshall for sparse graphs

## Minimum Spanning Tree Algorithms

### Prim's Algorithm
Finds a minimum spanning tree using a greedy approach.
- Time: O(E log V) with binary heap
- Space: O(V)
- Works only with undirected graphs

### Kruskal's Algorithm
Finds a minimum spanning tree using disjoint sets.
- Time: O(E log E)
- Space: O(V)
- Works only with undirected graphs

## Strongly Connected Components

### Tarjan's Algorithm
Finds strongly connected components in a single DFS pass.
- Time: O(V + E)
- Space: O(V)
- More space-efficient than Kosaraju's

### Kosaraju's Algorithm
Finds strongly connected components using two DFS passes.
- Time: O(V + E)
- Space: O(V)
- Simpler to understand than Tarjan's

## Graph Properties and Analysis

### Warshall's Algorithm
Computes the transitive closure of a graph.
- Time: O(V³)
- Space: O(V²)
- Works with both directed and undirected graphs

### Topological Sort
Sorts vertices in a directed acyclic graph.
- Time: O(V + E)
- Space: O(V)
- Requires: No cycles (DAG)

# Examples

```rust
use blocks_cs_graph::algorithms::dijkstra;
use blocks_cs_graph::graph::Graph;

// Create a weighted directed graph
let mut graph = Graph::new();
graph.add_edge(0, 1, 4.0);
graph.add_edge(0, 2, 2.0);
graph.add_edge(1, 3, 3.0);
graph.add_edge(2, 1, 1.0);
graph.add_edge(2, 3, 5.0);

// Find shortest paths from vertex 0
let shortest_paths = dijkstra::shortest_paths(&graph, &0)
    .expect("Graph should be valid for Dijkstra's algorithm");

assert_eq!(shortest_paths[&3], Some(6.0)); // Path 0->2->1->3 with total weight 6
```
*/

pub mod dijkstra;

pub use dijkstra::shortest_paths as dijkstra;