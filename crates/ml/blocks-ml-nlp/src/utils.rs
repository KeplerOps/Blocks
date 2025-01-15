//! Common utilities for NLP algorithms

/// Splits text into tokens (words) while preserving certain punctuation marks
pub fn tokenize(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|s| s.to_string())
        .collect()
}

/// Joins tokens back into text
pub fn detokenize(tokens: &[String]) -> String {
    tokens.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "hello world";
        let tokens = tokenize(text);
        assert_eq!(tokens, vec!["hello", "world"]);
    }

    #[test]
    fn test_detokenize() {
        let tokens = vec!["hello".to_string(), "world".to_string()];
        let text = detokenize(&tokens);
        assert_eq!(text, "hello world");
    }
}