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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let errors = vec![
            (NLPError::ParseError("invalid syntax".to_string()), "Parse error: invalid syntax"),
            (NLPError::GrammarError("invalid rule".to_string()), "Grammar error: invalid rule"),
            (NLPError::InputError("empty input".to_string()), "Input error: empty input"),
            (NLPError::ModelError("invalid params".to_string()), "Model error: invalid params"),
            (NLPError::TrainingError("no data".to_string()), "Training error: no data"),
        ];

        for (error, expected) in errors {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn test_error_debug() {
        let error = NLPError::ParseError("test".to_string());
        assert!(format!("{:?}", error).contains("ParseError"));
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        let err_result: Result<i32> = Err(NLPError::ParseError("test".to_string()));
        
        assert!(ok_result.is_ok());
        assert!(err_result.is_err());
    }
}