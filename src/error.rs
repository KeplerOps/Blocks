use thiserror::Error;
use std::fmt::{Display, Formatter};
use std::collections::TryReserveError;

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
        /// The source error if available
        #[source]
        source: Option<TryReserveError>,
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
    pub(crate) fn allocation_failed(reason: impl Display, source: Option<TryReserveError>) -> Self {
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
        let mut v: Vec<i32> = Vec::new();
        let err = v.try_reserve(usize::MAX).unwrap_err();
        let err = SortError::allocation_failed("failed to allocate", Some(err));
        assert!(err.source().is_some());

        let err = SortError::recursion_limit_exceeded(10, 5);
        assert!(err.source().is_none());
    }

    #[test]
    fn test_error_display() {
        let mut v: Vec<i32> = Vec::new();
        let reserve_err = v.try_reserve(usize::MAX).unwrap_err();
        let err = SortError::allocation_failed("failed to allocate", Some(reserve_err));
        assert!(err.to_string().contains("failed to allocate"));
    }
}

/// Custom error type for search operations
#[derive(Error, Debug, PartialEq)]
pub enum SearchError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(String),
    
    #[error("Operation not supported: {0}")]
    Unsupported(String),
}

/// Result type for search operations
pub type Result<T> = std::result::Result<T, SearchError>;

/// Errors that can occur during string algorithm operations.
#[derive(Debug, Error)]
pub enum StringError {
    /// The pattern is empty.
    #[error("Pattern cannot be empty")]
    EmptyPattern,

    /// The pattern is longer than the text.
    #[error("Pattern length {pattern_len} is longer than text length {text_len}")]
    PatternTooLong {
        /// Length of the pattern
        pattern_len: usize,
        /// Length of the text
        text_len: usize,
    },

    /// Failed to allocate memory for algorithm operations.
    #[error("Failed to allocate memory: {reason}")]
    AllocationFailed {
        /// The reason for the allocation failure
        reason: String,
    },

    /// Invalid input parameters provided.
    #[error("Invalid input: {reason}")]
    InvalidInput {
        /// Description of why the input is invalid
        reason: String,
    },
}

/// A specialized Result type for string algorithm operations.
pub type Result<T> = std::result::Result<T, StringError>;

impl StringError {
    /// Creates a new EmptyPattern error.
    pub(crate) fn empty_pattern() -> Self {
        Self::EmptyPattern
    }

    /// Creates a new PatternTooLong error.
    pub(crate) fn pattern_too_long(pattern_len: usize, text_len: usize) -> Self {
        Self::PatternTooLong {
            pattern_len,
            text_len,
        }
    }

    /// Creates a new AllocationFailed error.
    #[allow(dead_code)]
    pub(crate) fn allocation_failed(reason: impl Display) -> Self {
        Self::AllocationFailed {
            reason: reason.to_string(),
        }
    }

    /// Creates a new InvalidInput error.
    #[allow(dead_code)]
    pub(crate) fn invalid_input(reason: impl Display) -> Self {
        Self::InvalidInput {
            reason: reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_messages() {
        let err = StringError::empty_pattern();
        assert_eq!(err.to_string(), "Pattern cannot be empty");

        let err = StringError::pattern_too_long(10, 5);
        assert_eq!(
            err.to_string(),
            "Pattern length 10 is longer than text length 5"
        );

        let err = StringError::allocation_failed("failed to allocate buffer");
        assert_eq!(err.to_string(), "Failed to allocate memory: failed to allocate buffer");

        let err = StringError::invalid_input("invalid UTF-8");
        assert_eq!(err.to_string(), "Invalid input: invalid UTF-8");
    }
}
