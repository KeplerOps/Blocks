use thiserror::Error;

/// Result type for ML operations
pub type Result<T> = std::result::Result<T, Error>;

/// Error types for ML operations
#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid input dimensions: expected {expected:?}, got {got:?}")]
    InvalidDimensions {
        expected: Vec<usize>,
        got: Vec<usize>,
    },

    #[error("Invalid parameter value: {0}")]
    InvalidParameter(String),

    #[error("Convergence failed: {0}")]
    ConvergenceFailed(String),

    #[error("Numerical error: {0}")]
    NumericalError(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}