use std::fmt::Display;
use thiserror::Error;

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
