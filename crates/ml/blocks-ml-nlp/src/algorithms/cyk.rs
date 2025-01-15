use std::collections::{HashMap, HashSet};
use crate::error::{Result, NLPError};

/// A rule in Chomsky Normal Form (CNF)
#[derive(Debug, Clone)]
enum CNFRule {
    /// A → BC where A, B, C are non-terminals
    Binary(String, String, String),
    /// A → a where A is a non-terminal and a is a terminal
    Terminal(String, String),
}

/// CYK Parser for context-free grammars in Chomsky Normal Form
///
/// # Algorithm Complexity
/// - Time Complexity: O(n³|G|), where n is the length of the input string and |G| is the size of the grammar
/// - Space Complexity: O(n²|N|), where n is the length of the input string and |N| is the number of non-terminals
///
/// # Memory Usage
/// The algorithm uses a dynamic programming table of size n × n × |N|, where:
/// - n is the length of the input string
/// - |N| is the number of non-terminals in the grammar
///
/// For very long sentences or grammars with many non-terminals, memory usage can be significant.
///
/// # Limitations
/// - Grammar must be in Chomsky Normal Form (CNF)
/// - Does not handle empty productions (ε-productions)
/// - Does not produce parse trees (only recognizes if a string is in the language)
#[derive(Debug)]
pub struct CYKParser {
    /// Rules of the grammar in CNF
    rules: Vec<CNFRule>,
    /// Set of non-terminal symbols
    non_terminals: HashSet<String>,
    /// Start symbol of the grammar
    start_symbol: String,
}

impl CYKParser {
    /// Creates a new CYK parser from a grammar in CNF
    ///
    /// # Arguments
    /// * `rules` - Vector of tuples (head, body) where head is a non-terminal
    ///   and body is either a terminal or two non-terminals
    ///
    /// # Returns
    /// * `Result<CYKParser>` - New parser instance or error if grammar is invalid
    ///
    /// # Example
    /// ```
    /// use blocks_ml_nlp::algorithms::CYKParser;
    ///
    /// let grammar = vec![
    ///     ("S", vec!["NP VP"]),
    ///     ("NP", vec!["Det N"]),
    ///     ("VP", vec!["V NP"]),
    ///     ("Det", vec!["the"]),
    ///     ("N", vec!["cat"]),
    ///     ("V", vec!["saw"]),
    /// ];
    ///
    /// let parser = CYKParser::new(grammar).unwrap();
    /// ```
    pub fn new(grammar: Vec<(&str, Vec<&str>)>) -> Result<Self> {
        let mut rules = Vec::new();
        let mut non_terminals = HashSet::new();
        let start_symbol = grammar.first()
            .ok_or_else(|| NLPError::GrammarError("Empty grammar".to_string()))?
            .0.to_string();

        for (head, bodies) in grammar {
            non_terminals.insert(head.to_string());
            
            for body in bodies {
                let parts: Vec<&str> = body.split_whitespace().collect();
                match parts.len() {
                    1 => rules.push(CNFRule::Terminal(
                        head.to_string(),
                        parts[0].to_string()
                    )),
                    2 => {
                        non_terminals.insert(parts[0].to_string());
                        non_terminals.insert(parts[1].to_string());
                        rules.push(CNFRule::Binary(
                            head.to_string(),
                            parts[0].to_string(),
                            parts[1].to_string()
                        ));
                    },
                    _ => return Err(NLPError::GrammarError(
                        format!("Invalid rule: {} → {}", head, body)
                    )),
                }
            }
        }

        Ok(CYKParser {
            rules,
            non_terminals,
            start_symbol,
        })
    }

    /// Parses a sentence using the CYK algorithm
    ///
    /// # Arguments
    /// * `sentence` - Space-separated string of terminals to parse
    ///
    /// # Returns
    /// * `Result<bool>` - True if sentence is in the language, false otherwise
    ///
    /// # Example
    /// ```
    /// # use blocks_ml_nlp::algorithms::CYKParser;
    /// # let grammar = vec![
    /// #     ("S", vec!["NP VP"]),
    /// #     ("NP", vec!["Det N"]),
    /// #     ("VP", vec!["V NP"]),
    /// #     ("Det", vec!["the"]),
    /// #     ("N", vec!["cat"]),
    /// #     ("V", vec!["saw"]),
    /// # ];
    /// # let parser = CYKParser::new(grammar).unwrap();
    /// let result = parser.parse("the cat saw the cat").unwrap();
    /// assert!(result);
    /// ```
    pub fn parse(&self, sentence: &str) -> Result<bool> {
        let words: Vec<&str> = sentence.split_whitespace().collect();
        let n = words.len();
        if n == 0 {
            return Ok(false);
        }

        // table[i][j] contains the set of non-terminals that can generate
        // the substring from position i to j (inclusive)
        let mut table = vec![vec![HashSet::new(); n]; n];

        // Fill in the base cases (diagonal)
        for i in 0..n {
            for rule in &self.rules {
                if let CNFRule::Terminal(head, terminal) = rule {
                    if terminal == words[i] {
                        table[i][i].insert(head.clone());
                    }
                }
            }
        }

        // Fill in the rest of the table
        for l in 2..=n {  // length of span
            for i in 0..=n-l {  // start of span
                let j = i + l - 1;  // end of span
                for k in i..j {  // partition point
                    for rule in &self.rules {
                        if let CNFRule::Binary(head, left, right) = rule {
                            if table[i][k].contains(left) && table[k+1][j].contains(right) {
                                table[i][j].insert(head.clone());
                            }
                        }
                    }
                }
            }
        }

        Ok(table[0][n-1].contains(&self.start_symbol))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_grammar() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("S", vec!["NP VP"]),
            ("NP", vec!["Det N", "NP PP"]),
            ("VP", vec!["V NP", "VP PP"]),
            ("PP", vec!["P NP"]),
            ("Det", vec!["the", "a"]),
            ("N", vec!["cat", "dog", "mouse"]),
            ("V", vec!["chased", "saw"]),
            ("P", vec!["with", "in"]),
        ]
    }

    fn create_unicode_grammar() -> Vec<(&'static str, Vec<&'static str>)> {
        vec![
            ("S", vec!["NP VP"]),
            ("NP", vec!["Det N"]),
            ("VP", vec!["V NP"]),
            ("Det", vec!["el", "la"]),
            ("N", vec!["niño", "niña", "perro"]),
            ("V", vec!["vio", "amó"]),
        ]
    }

    #[test]
    fn test_empty_sentence() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        assert!(!parser.parse("").unwrap());
    }

    #[test]
    fn test_simple_sentence() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        assert!(parser.parse("the cat saw a mouse").unwrap());
    }

    #[test]
    fn test_complex_sentence() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        assert!(parser.parse("the cat with a mouse saw the dog").unwrap());
    }

    #[test]
    fn test_invalid_sentence() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        assert!(!parser.parse("cat the saw mouse").unwrap());
    }

    #[test]
    fn test_invalid_word() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        assert!(!parser.parse("the cat ate the mouse").unwrap());
    }

    #[test]
    fn test_empty_grammar() {
        let result = CYKParser::new(vec![]);
        assert!(matches!(result, Err(NLPError::GrammarError(_))));
    }

    #[test]
    fn test_invalid_rule() {
        let result = CYKParser::new(vec![
            ("S", vec!["A B C"]),  // Invalid: more than 2 symbols
        ]);
        assert!(matches!(result, Err(NLPError::GrammarError(_))));
    }

    #[test]
    fn test_recursive_grammar() {
        let grammar = vec![
            ("S", vec!["NP VP"]),
            ("NP", vec!["NP PP", "Det N"]),  // Recursive rule
            ("PP", vec!["P NP"]),
            ("Det", vec!["the"]),
            ("N", vec!["cat"]),
            ("V", vec!["saw"]),
            ("P", vec!["with"]),
        ];
        let parser = CYKParser::new(grammar).unwrap();
        assert!(parser.parse("the cat with the cat saw the cat").unwrap());
    }

    #[test]
    fn test_large_sentence() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        let sentence = "the cat saw the dog with the mouse in the cat with a dog";
        assert!(parser.parse(sentence).unwrap());
    }

    #[test]
    fn test_all_terminals() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        let mut all_valid = true;
        for word in ["the", "a", "cat", "dog", "mouse", "chased", "saw", "with", "in"] {
            if parser.parse(word).unwrap() {
                all_valid = false;
                break;
            }
        }
        assert!(all_valid, "Single terminals should not form valid sentences");
    }

    #[test]
    fn test_unicode_simple() {
        let parser = CYKParser::new(create_unicode_grammar()).unwrap();
        assert!(parser.parse("el niño vio la niña").unwrap());
    }

    #[test]
    fn test_unicode_invalid() {
        let parser = CYKParser::new(create_unicode_grammar()).unwrap();
        assert!(!parser.parse("el vio niño la").unwrap());
    }

    #[test]
    fn test_very_long_sentence() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        let sentence = "the cat saw a dog with a mouse in the house with the cat in a tree with the bird in the sky";
        assert!(parser.parse(sentence).unwrap());
    }

    #[test]
    fn test_deep_recursion() {
        let grammar = vec![
            ("S", vec!["A B"]),
            ("A", vec!["A A", "a"]),  // Allows for deep left recursion
            ("B", vec!["b"]),
        ];
        let parser = CYKParser::new(grammar).unwrap();
        
        // Create a sentence with many 'a's followed by 'b'
        let many_as: String = "a ".repeat(100);
        let sentence = format!("{} b", many_as);
        assert!(parser.parse(&sentence).unwrap());
    }

    #[test]
    fn test_ambiguous_grammar() {
        let grammar = vec![
            ("S", vec!["A B", "B A"]),
            ("A", vec!["a"]),
            ("B", vec!["b"]),
        ];
        let parser = CYKParser::new(grammar).unwrap();
        assert!(parser.parse("a b").unwrap());
        assert!(parser.parse("b a").unwrap());
    }

    #[test]
    fn test_empty_production() {
        let result = CYKParser::new(vec![
            ("S", vec![""]),  // Empty production
        ]);
        assert!(matches!(result, Err(NLPError::GrammarError(_))));
    }

    #[test]
    fn test_whitespace_handling() {
        let parser = CYKParser::new(create_test_grammar()).unwrap();
        assert!(parser.parse("the   cat   saw   a   mouse").unwrap());
        assert!(parser.parse("\tthe\ncat\rsaw\ta\nmouse").unwrap());
    }
}