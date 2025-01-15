use crate::error::{Result, SearchError};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// Represents a graph using an adjacency list.
#[derive(Debug, Clone)]
pub struct Graph<T> {
    /// Adjacency list representation of the graph
    edges: HashMap<T, Vec<T>>,
}

impl<T> Graph<T>
where
    T: Eq + Hash + Clone,
{
    /// Creates a new empty graph
    pub fn new() -> Self {
        Graph {
            edges: HashMap::new(),
        }
    }

    /// Adds a vertex to the graph
    pub fn add_vertex(&mut self, vertex: T) {
        self.edges.entry(vertex).or_insert_with(Vec::new);
    }

    /// Adds a directed edge from source to destination
    pub fn add_edge(&mut self, source: T, destination: T) {
        self.edges.entry(source.clone()).or_default().push(destination.clone());
        // Ensure the destination vertex exists in the graph
        self.edges.entry(destination).or_default();
    }

    /// Performs a depth-first search to find a path to the target vertex
    ///
    /// # Arguments
    /// * `start` - The vertex to start the search from
    /// * `target` - The vertex to search for
    ///
    /// # Returns
    /// * `Ok(Some(path))` - A vector representing the path from start to target
    /// * `Ok(None)` - Target vertex not found
    /// * `Err(SearchError)` - An error occurred during the search
    ///
    /// # Examples
    /// ```
    /// use blocks_cs_search::algorithms::dfs::Graph;
    ///
    /// let mut graph = Graph::new();
    /// graph.add_edge(1, 2);
    /// graph.add_edge(2, 3);
    /// graph.add_edge(1, 4);
    ///
    /// assert_eq!(graph.search(&1, &3), Ok(Some(vec![1, 2, 3])));
    /// assert_eq!(graph.search(&1, &5), Ok(None));
    /// ```
    pub fn search(&self, start: &T, target: &T) -> Result<Option<Vec<T>>> {
        if !self.edges.contains_key(start) {
            return Err(SearchError::InvalidInput(
                "Start vertex not found in graph".to_string(),
            ));
        }

        let mut visited = HashSet::new();
        let mut path = Vec::new();
        
        if self.dfs_recursive(start, target, &mut visited, &mut path) {
            Ok(Some(path))
        } else {
            Ok(None)
        }
    }

    /// Recursive helper function for depth-first search
    fn dfs_recursive(
        &self,
        current: &T,
        target: &T,
        visited: &mut HashSet<T>,
        path: &mut Vec<T>,
    ) -> bool {
        visited.insert(current.clone());
        path.push(current.clone());

        if current == target {
            return true;
        }

        if let Some(neighbors) = self.edges.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_recursive(neighbor, target, visited, path) {
                        return true;
                    }
                }
            }
        }

        path.pop();
        false
    }

    /// Performs an iterative depth-first search using a stack
    ///
    /// # Arguments
    /// * `start` - The vertex to start the search from
    /// * `target` - The vertex to search for
    ///
    /// # Returns
    /// * `Ok(Some(path))` - A vector representing the path from start to target
    /// * `Ok(None)` - Target vertex not found
    /// * `Err(SearchError)` - An error occurred during the search
    pub fn search_iterative(&self, start: &T, target: &T) -> Result<Option<Vec<T>>> {
        if !self.edges.contains_key(start) {
            return Err(SearchError::InvalidInput(
                "Start vertex not found in graph".to_string(),
            ));
        }

        let mut stack = vec![(start.clone(), vec![start.clone()])];
        let mut visited = HashSet::new();
        visited.insert(start.clone());

        while let Some((current, path)) = stack.pop() {
            if &current == target {
                return Ok(Some(path));
            }

            if let Some(neighbors) = self.edges.get(&current) {
                for neighbor in neighbors.iter().rev() {  // Reverse to maintain same order as recursive
                    if !visited.contains(neighbor) {
                        visited.insert(neighbor.clone());
                        let mut new_path = path.clone();
                        new_path.push(neighbor.clone());
                        stack.push((neighbor.clone(), new_path));
                    }
                }
            }
        }

        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_graph() {
        let graph: Graph<i32> = Graph::new();
        assert!(matches!(
            graph.search(&1, &2),
            Err(SearchError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_single_vertex() {
        let mut graph = Graph::new();
        graph.add_vertex(1);
        assert_eq!(graph.search(&1, &1), Ok(Some(vec![1])));
        assert_eq!(graph.search_iterative(&1, &1), Ok(Some(vec![1])));
    }

    #[test]
    fn test_direct_edge() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        assert_eq!(graph.search(&1, &2), Ok(Some(vec![1, 2])));
        assert_eq!(graph.search_iterative(&1, &2), Ok(Some(vec![1, 2])));
    }

    #[test]
    fn test_path_not_found() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        assert_eq!(graph.search(&1, &4), Ok(None));
        assert_eq!(graph.search_iterative(&1, &4), Ok(None));
    }

    #[test]
    fn test_complex_path() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(2, 4);
        graph.add_edge(3, 5);
        graph.add_edge(4, 5);

        let path = graph.search(&1, &5).unwrap().unwrap();
        assert!(path.starts_with(&[1, 2]));
        assert_eq!(*path.last().unwrap(), 5);

        let path_iter = graph.search_iterative(&1, &5).unwrap().unwrap();
        assert!(path_iter.starts_with(&[1, 2]));
        assert_eq!(*path_iter.last().unwrap(), 5);
    }

    #[test]
    fn test_cyclic_graph() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 1);
        graph.add_edge(2, 4);

        assert_eq!(graph.search(&1, &4), Ok(Some(vec![1, 2, 4])));
        assert_eq!(graph.search_iterative(&1, &4), Ok(Some(vec![1, 2, 4])));
    }

    #[test]
    fn test_with_strings() {
        let mut graph = Graph::new();
        graph.add_edge("A", "B");
        graph.add_edge("B", "C");
        graph.add_edge("A", "D");

        assert_eq!(
            graph.search(&"A", &"C"),
            Ok(Some(vec!["A", "B", "C"]))
        );
        assert_eq!(
            graph.search_iterative(&"A", &"C"),
            Ok(Some(vec!["A", "B", "C"]))
        );
    }

    #[test]
    fn test_disconnected_components() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(3, 4);

        assert_eq!(graph.search(&1, &4), Ok(None));
        assert_eq!(graph.search_iterative(&1, &4), Ok(None));
    }

    #[test]
    fn test_invalid_start_vertex() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);

        assert!(matches!(
            graph.search(&3, &2),
            Err(SearchError::InvalidInput(_))
        ));
        assert!(matches!(
            graph.search_iterative(&3, &2),
            Err(SearchError::InvalidInput(_))
        ));
    }

    #[test]
    fn test_multiple_paths() {
        let mut graph = Graph::new();
        graph.add_edge(1, 2);
        graph.add_edge(2, 4);
        graph.add_edge(1, 3);
        graph.add_edge(3, 4);

        // Both paths are valid, but DFS will find one of them
        let path = graph.search(&1, &4).unwrap().unwrap();
        assert!(
            path == vec![1, 2, 4] || path == vec![1, 3, 4],
            "Path should be either [1,2,4] or [1,3,4]"
        );

        let path_iter = graph.search_iterative(&1, &4).unwrap().unwrap();
        assert!(
            path_iter == vec![1, 2, 4] || path_iter == vec![1, 3, 4],
            "Path should be either [1,2,4] or [1,3,4]"
        );
    }
}