/*!
This crate provides a collection of graph algorithms implemented in Rust.

Each algorithm is implemented with a focus on:
- Performance optimizations
- Memory efficiency
- Comprehensive testing
- Clear documentation
- Modern Rust idioms

# Available Algorithms

## Shortest Path Algorithms
- [`Dijkstra`](mod@algorithms::dijkstra): Single-source shortest path for graphs with non-negative weights
- [`BellmanFord`](mod@algorithms::bellman_ford): Single-source shortest path that handles negative weights
- [`FloydWarshall`](mod@algorithms::floyd_warshall): All-pairs shortest path algorithm
- [`Johnson`](mod@algorithms::johnson): All-pairs shortest path optimized for sparse graphs

## Minimum Spanning Tree Algorithms
- [`Prim`](mod@algorithms::prim): Minimum spanning tree using a greedy approach
- [`Kruskal`](mod@algorithms::kruskal): Minimum spanning tree using disjoint sets

## Strongly Connected Components
- [`Tarjan`](mod@algorithms::tarjan): Find strongly connected components using Tarjan's algorithm
- [`Kosaraju`](mod@algorithms::kosaraju): Find strongly connected components using Kosaraju's algorithm

## Graph Properties and Analysis
- [`Warshall`](mod@algorithms::warshall): Compute transitive closure of a graph
- [`TopologicalSort`](mod@algorithms::topological_sort): Sort vertices in a directed acyclic graph

# Usage Example

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

# Features
- Generic implementations that work with any numeric weight type
- Support for both directed and undirected graphs
- Comprehensive test suites including edge cases
- Detailed documentation with complexity analysis
*/

pub mod algorithms;
pub mod error;
pub mod graph;

pub use error::{Result, GraphError};
pub use graph::Graph;
