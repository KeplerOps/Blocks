use std::fmt::Display;
use thiserror::Error;

/// Errors that can occur during sorting operations.
#[derive(Debug, Error)]
pub enum SortError {
    /// The recursion depth exceeded the configured maximum.
    #[error("Recursion depth {depth} exceeded maximum allowed depth of {max_depth}")]
    RecursionLimitExceeded {
        /// The depth that was reached
        depth: usize,
        /// The maximum allowed depth
        max_depth: usize,
    },

    /// Failed to allocate memory for sorting operations.
    #[error("Failed to allocate memory: {reason}")]
    AllocationFailed {
        /// The reason for the allocation failure
        reason: String,
        /// The number of bytes that failed to allocate
        #[source]
        source: Option<std::alloc::AllocError>,
    },

    /// A parallel execution task failed.
    #[error("Parallel execution failed: {reason}")]
    ParallelExecutionFailed {
        /// The reason for the failure
        reason: String,
    },

    /// The input slice was too large to sort.
    #[error("Input slice length {length} exceeds maximum supported length of {max_length}")]
    InputTooLarge {
        /// The actual length of the input
        length: usize,
        /// The maximum supported length
        max_length: usize,
    },
}

/// A specialized Result type for sorting operations.
pub type Result<T> = std::result::Result<T, SortError>;

impl SortError {
    /// Creates a new RecursionLimitExceeded error.
    pub(crate) fn recursion_limit_exceeded(depth: usize, max_depth: usize) -> Self {
        Self::RecursionLimitExceeded { depth, max_depth }
    }

    /// Creates a new AllocationFailed error.
    pub(crate) fn allocation_failed(reason: impl Display, source: Option<std::alloc::AllocError>) -> Self {
        Self::AllocationFailed {
            reason: reason.to_string(),
            source,
        }
    }

    /// Creates a new ParallelExecutionFailed error.
    pub(crate) fn parallel_execution_failed(reason: impl Display) -> Self {
        Self::ParallelExecutionFailed {
            reason: reason.to_string(),
        }
    }

    /// Creates a new InputTooLarge error.
    pub(crate) fn input_too_large(length: usize, max_length: usize) -> Self {
        Self::InputTooLarge {
            length,
            max_length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = SortError::recursion_limit_exceeded(100, 50);
        assert_eq!(
            err.to_string(),
            "Recursion depth 100 exceeded maximum allowed depth of 50"
        );

        let err = SortError::allocation_failed("failed to allocate buffer", None);
        assert_eq!(err.to_string(), "Failed to allocate memory: failed to allocate buffer");

        let err = SortError::parallel_execution_failed("thread panic");
        assert_eq!(err.to_string(), "Parallel execution failed: thread panic");

        let err = SortError::input_too_large(1_000_000, 100_000);
        assert_eq!(
            err.to_string(),
            "Input slice length 1000000 exceeds maximum supported length of 100000"
        );
    }

    #[test]
    fn test_error_sources() {
        let err = SortError::allocation_failed(
            "failed to allocate",
            Some(std::alloc::AllocError),
        );
        assert!(err.source().is_some());

        let err = SortError::recursion_limit_exceeded(10, 5);
        assert!(err.source().is_none());
    }
}