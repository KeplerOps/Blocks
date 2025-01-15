use thiserror::Error;

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