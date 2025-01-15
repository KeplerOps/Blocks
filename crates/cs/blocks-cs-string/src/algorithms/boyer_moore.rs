use crate::error::{Result, StringError};
use std::collections::HashMap;

/// Preprocesses the pattern to build the bad character rule table.
/// This rule determines how far to shift the pattern when a character mismatch occurs.
///
/// # Arguments
/// * `pattern` - The pattern to preprocess
///
/// # Returns
/// * `HashMap<u8, usize>` - Maps each character to its rightmost position in the pattern
fn build_bad_char_table(pattern: &[u8]) -> HashMap<u8, usize> {
    let mut bad_char = HashMap::new();
    for (i, &c) in pattern.iter().enumerate() {
        bad_char.insert(c, i);
    }
    bad_char
}

/// Preprocesses the pattern to build the good suffix rule table.
/// This rule determines how far to shift the pattern based on the longest matching suffix.
///
/// # Arguments
/// * `pattern` - The pattern to preprocess
///
/// # Returns
/// * `Vec<usize>` - The good suffix shift table
fn build_good_suffix_table(pattern: &[u8]) -> Vec<usize> {
    let m = pattern.len();
    let mut suffix = vec![0; m];
    let mut g = vec![m; m + 1];
    let mut f = vec![0; m + 1];
    
    // Case 1: suffix matches prefix
    let mut i = m;
    let mut j = m + 1;
    f[i] = j;
    while i > 0 {
        while j <= m && pattern[i - 1] != pattern[j - 1] {
            if g[j] == m {
                g[j] = j - i;
            }
            j = f[j];
        }
        i -= 1;
        j -= 1;
        f[i] = j;
    }

    // Case 2: only proper suffixes
    j = f[0];
    for i in 0..=m {
        if g[i] == m {
            g[i] = j;
        }
        if i == j {
            j = f[j];
        }
    }

    // Build the final good suffix table
    for i in 0..m {
        suffix[i] = g[i + 1];
    }
    
    suffix
}

/// Finds all occurrences of a pattern in the given text using the Boyer-Moore algorithm.
///
/// # Arguments
/// * `text` - The text to search in
/// * `pattern` - The pattern to search for
///
/// # Returns
/// * `Result<Vec<usize>>` - A vector containing all starting positions where the pattern occurs in the text
///
/// # Errors
/// * `StringError::EmptyPattern` if the pattern is empty
/// * `StringError::PatternTooLong` if pattern length exceeds text length
///
/// # Example
/// ```
/// use blocks_cs_string::algorithms::boyer_moore;
///
/// let text = "AABAACAADAABAAABAA";
/// let pattern = "AABA";
/// let positions = boyer_moore::find_all(text, pattern).unwrap();
/// assert_eq!(positions, vec![0, 9, 13]);
/// ```
pub fn find_all(text: impl AsRef<[u8]>, pattern: impl AsRef<[u8]>) -> Result<Vec<usize>> {
    let text = text.as_ref();
    let pattern = pattern.as_ref();

    // Validate inputs
    if pattern.is_empty() {
        return Err(StringError::empty_pattern());
    }
    if pattern.len() > text.len() {
        return Err(StringError::pattern_too_long(pattern.len(), text.len()));
    }

    let m = pattern.len();
    let n = text.len();
    let mut matches = Vec::new();

    // Edge case: empty text
    if n == 0 {
        return Ok(matches);
    }

    // Preprocess pattern
    let bad_char = build_bad_char_table(pattern);
    let good_suffix = build_good_suffix_table(pattern);

    // Search phase
    let mut i = m - 1; // Index in text
    let mut j; // Index in pattern
    while i < n {
        j = m - 1;
        let mut k = i;
        
        // Try to match pattern from right to left
        while j < m && pattern[j] == text[k] {
            if j == 0 {
                matches.push(k);
                break;
            }
            j -= 1;
            k -= 1;
        }

        // Calculate shift distance using both rules and take maximum
        let bad_char_shift = match bad_char.get(&text[i]) {
            Some(&pos) => m - 1 - pos,
            None => m,
        };
        let good_suffix_shift = if j < m { good_suffix[j] } else { 1 };
        
        i += std::cmp::max(bad_char_shift, good_suffix_shift);
    }

    Ok(matches)
}

/// Finds the first occurrence of a pattern in the given text using the Boyer-Moore algorithm.
///
/// # Arguments
/// * `text` - The text to search in
/// * `pattern` - The pattern to search for
///
/// # Returns
/// * `Result<Option<usize>>` - The starting position of the first occurrence if found, None otherwise
///
/// # Errors
/// * `StringError::EmptyPattern` if the pattern is empty
/// * `StringError::PatternTooLong` if pattern length exceeds text length
///
/// # Example
/// ```
/// use blocks_cs_string::algorithms::boyer_moore;
///
/// let text = "AABAACAADAABAAABAA";
/// let pattern = "AABA";
/// let position = boyer_moore::find_first(text, pattern).unwrap();
/// assert_eq!(position, Some(0));
/// ```
pub fn find_first(text: impl AsRef<[u8]>, pattern: impl AsRef<[u8]>) -> Result<Option<usize>> {
    let text = text.as_ref();
    let pattern = pattern.as_ref();

    // Validate inputs
    if pattern.is_empty() {
        return Err(StringError::empty_pattern());
    }
    if pattern.len() > text.len() {
        return Err(StringError::pattern_too_long(pattern.len(), text.len()));
    }

    let m = pattern.len();
    let n = text.len();

    // Edge case: empty text
    if n == 0 {
        return Ok(None);
    }

    // Preprocess pattern
    let bad_char = build_bad_char_table(pattern);
    let good_suffix = build_good_suffix_table(pattern);

    // Search phase
    let mut i = m - 1; // Index in text
    let mut j; // Index in pattern
    while i < n {
        j = m - 1;
        let mut k = i;
        
        // Try to match pattern from right to left
        while j < m && pattern[j] == text[k] {
            if j == 0 {
                return Ok(Some(k));
            }
            j -= 1;
            k -= 1;
        }

        // Calculate shift distance using both rules and take maximum
        let bad_char_shift = match bad_char.get(&text[i]) {
            Some(&pos) => m - 1 - pos,
            None => m,
        };
        let good_suffix_shift = if j < m { good_suffix[j] } else { 1 };
        
        i += std::cmp::max(bad_char_shift, good_suffix_shift);
    }

    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern() {
        let text = "hello";
        let pattern = "";
        assert!(matches!(
            find_all(text, pattern),
            Err(StringError::EmptyPattern)
        ));
    }

    #[test]
    fn test_pattern_too_long() {
        let text = "hi";
        let pattern = "hello";
        assert!(matches!(
            find_all(text, pattern),
            Err(StringError::PatternTooLong { .. })
        ));
    }

    #[test]
    fn test_pattern_not_found() {
        let text = "hello world";
        let pattern = "xyz";
        assert_eq!(find_all(text, pattern).unwrap(), Vec::<usize>::new());
        assert_eq!(find_first(text, pattern).unwrap(), None);
    }

    #[test]
    fn test_single_match() {
        let text = "hello world";
        let pattern = "world";
        assert_eq!(find_all(text, pattern).unwrap(), vec![6]);
        assert_eq!(find_first(text, pattern).unwrap(), Some(6));
    }

    #[test]
    fn test_multiple_matches() {
        let text = "AABAACAADAABAAABAA";
        let pattern = "AABA";
        assert_eq!(find_all(text, pattern).unwrap(), vec![0, 9, 13]);
        assert_eq!(find_first(text, pattern).unwrap(), Some(0));
    }

    #[test]
    fn test_overlapping_matches() {
        let text = "AAAAA";
        let pattern = "AA";
        assert_eq!(find_all(text, pattern).unwrap(), vec![0, 1, 2, 3]);
        assert_eq!(find_first(text, pattern).unwrap(), Some(0));
    }

    #[test]
    fn test_match_at_start() {
        let text = "hello world";
        let pattern = "hello";
        assert_eq!(find_all(text, pattern).unwrap(), vec![0]);
        assert_eq!(find_first(text, pattern).unwrap(), Some(0));
    }

    #[test]
    fn test_match_at_end() {
        let text = "hello world";
        let pattern = "world";
        assert_eq!(find_all(text, pattern).unwrap(), vec![6]);
        assert_eq!(find_first(text, pattern).unwrap(), Some(6));
    }

    #[test]
    fn test_unicode_text() {
        let text = "Hello 世界!";
        let pattern = "世界";
        assert_eq!(find_all(text.as_bytes(), pattern.as_bytes()).unwrap(), vec![6]);
        assert_eq!(find_first(text.as_bytes(), pattern.as_bytes()).unwrap(), Some(6));
    }

    #[test]
    fn test_empty_text() {
        let text = "";
        let pattern = "a";
        assert!(matches!(
            find_all(text, pattern),
            Err(StringError::PatternTooLong { .. })
        ));
    }

    #[test]
    fn test_bad_char_rule() {
        let text = "ABCBAB";
        let pattern = "BAB";
        let bad_char = build_bad_char_table(pattern.as_bytes());
        assert_eq!(bad_char.get(&b'B'), Some(&2));
        assert_eq!(bad_char.get(&b'A'), Some(&1));
    }

    #[test]
    fn test_good_suffix_rule() {
        let pattern = "BAOBAB";
        let good_suffix = build_good_suffix_table(pattern.as_bytes());
        // Test the good suffix shifts for this pattern
        assert_eq!(good_suffix[0], 6); // No match
        assert_eq!(good_suffix[5], 1); // Single char match
    }
}
