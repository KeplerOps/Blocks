use std::fmt::{Debug, Display};
use thiserror::Error;

/// Errors that can occur during graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    /// The vertex does not exist in the graph.
    #[error("Vertex not found in graph")]
    VertexNotFound,

    /// The edge does not exist in the graph.
    #[error("Edge not found in graph")]
    EdgeNotFound,

    /// The graph contains a negative cycle.
    #[error("Negative cycle detected in graph")]
    NegativeCycle,

    /// The graph contains a cycle when the algorithm requires an acyclic graph.
    #[error("Cycle detected in graph")]
    CycleDetected,

    /// The graph is empty.
    #[error("Graph is empty")]
    EmptyGraph,

    /// The graph is not connected.
    #[error("Graph is not connected")]
    NotConnected,

    /// The operation failed due to invalid input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// The operation failed due to memory allocation.
    #[error("Memory allocation failed: {0}")]
    AllocationFailed(String),
}

/// A specialized Result type for graph operations.
pub type Result<T> = std::result::Result<T, GraphError>;

impl GraphError {
    /// Creates a new InvalidInput error.
    pub(crate) fn invalid_input(msg: impl Display) -> Self {
        Self::InvalidInput(msg.to_string())
    }

    /// Creates a new AllocationFailed error.
    pub(crate) fn allocation_failed(msg: impl Display) -> Self {
        Self::AllocationFailed(msg.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = GraphError::VertexNotFound;
        assert_eq!(err.to_string(), "Vertex not found in graph");

        let err = GraphError::EdgeNotFound;
        assert_eq!(err.to_string(), "Edge not found in graph");

        let err = GraphError::NegativeCycle;
        assert_eq!(err.to_string(), "Negative cycle detected in graph");

        let err = GraphError::CycleDetected;
        assert_eq!(err.to_string(), "Cycle detected in graph");

        let err = GraphError::EmptyGraph;
        assert_eq!(err.to_string(), "Graph is empty");

        let err = GraphError::NotConnected;
        assert_eq!(err.to_string(), "Graph is not connected");

        let err = GraphError::invalid_input("Invalid weight");
        assert_eq!(err.to_string(), "Invalid input: Invalid weight");

        let err = GraphError::allocation_failed("Failed to allocate memory");
        assert_eq!(err.to_string(), "Memory allocation failed: Failed to allocate memory");
    }
}