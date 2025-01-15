use std::error::Error;
use std::fmt;

/// Custom error type for NLP operations
#[derive(Debug)]
pub enum NLPError {
    /// Error when parsing fails
    ParseError(String),
    /// Error when grammar is invalid
    GrammarError(String),
    /// Error when input is invalid
    InputError(String),
    /// Error when model parameters are invalid
    ModelError(String),
    /// Error when training fails
    TrainingError(String),
}

impl fmt::Display for NLPError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NLPError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            NLPError::GrammarError(msg) => write!(f, "Grammar error: {}", msg),
            NLPError::InputError(msg) => write!(f, "Input error: {}", msg),
            NLPError::ModelError(msg) => write!(f, "Model error: {}", msg),
            NLPError::TrainingError(msg) => write!(f, "Training error: {}", msg),
        }
    }
}

impl Error for NLPError {}

/// Custom Result type for NLP operations
pub type Result<T> = std::result::Result<T, NLPError>;