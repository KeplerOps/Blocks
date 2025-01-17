use std::collections::{HashMap, VecDeque};

/// Configuration options for pattern matching behavior
#[derive(Debug, Clone)]
pub struct MatchConfig {
    /// Only match at word boundaries
    pub word_boundaries: bool,
    /// Only report the longest match at each position
    pub longest_match_only: bool,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            word_boundaries: false,
            longest_match_only: false,
        }
    }
}

/// Represents a match found by the Aho-Corasick algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Match {
    /// Index of the matched pattern in the original patterns vector
    pub pattern_index: usize,
    /// Start position of the match in the text
    pub start: usize,
    /// End position of the match in the text (exclusive)
    pub end: usize,
}

/// A node in the Aho-Corasick automaton's trie structure
#[derive(Debug)]
struct TrieNode {
    /// Children nodes indexed by character
    children: HashMap<char, usize>,
    /// Failure link to longest proper suffix state
    failure: Option<usize>,
    /// Bit vector where bit i is set if pattern i ends at this node
    output: u64,
    /// Depth in trie (for match position calculation)
    depth: usize,
}

impl TrieNode {
    fn new(depth: usize) -> Self {
        Self {
            children: HashMap::new(),
            failure: None,
            output: 0,
            depth,
        }
    }
}

/// An implementation of the Aho-Corasick string matching algorithm.
#[derive(Debug)]
pub struct AhoCorasick {
    /// All nodes in the automaton
    nodes: Vec<TrieNode>,
    /// Original patterns for reporting matches
    patterns: Vec<String>,
    /// Root node is always at index 0
    root: usize,
    /// Configuration for pattern matching behavior
    config: MatchConfig,
}

impl AhoCorasick {
    /// Creates a new Aho-Corasick automaton from the given patterns with default configuration.
    pub fn new(patterns: Vec<String>) -> Result<Self, String> {
        Self::with_config(patterns, MatchConfig::default())
    }

    /// Creates a new Aho-Corasick automaton with the specified configuration.
    pub fn with_config(patterns: Vec<String>, config: MatchConfig) -> Result<Self, String> {
        // Validate patterns
        if patterns.is_empty() {
            return Err("At least one pattern is required".to_string());
        }
        if patterns.iter().any(|p| p.is_empty()) {
            return Err("Empty patterns are not allowed".to_string());
        }
        if patterns.len() > 64 {
            return Err("Maximum of 64 patterns supported".to_string());
        }
        
        let mut ac = Self {
            nodes: vec![TrieNode::new(0)],
            patterns,
            root: 0,
            config,
        };

        // Build trie and failure links
        ac.build_trie()?;
        ac.build_failure_links();

        Ok(ac)
    }

    /// Builds the initial trie structure from the patterns
    fn build_trie(&mut self) -> Result<(), String> {
        // Insert each pattern into the trie
        for (pattern_idx, pattern) in self.patterns.iter().enumerate() {
            let mut current = self.root;

            // Follow/create path for each character
            for ch in pattern.chars() {
                current = match self.nodes[current].children.get(&ch) {
                    Some(&next) => next,
                    None => {
                        let next = self.nodes.len();
                        self.nodes.push(TrieNode::new(self.nodes[current].depth + 1));
                        self.nodes[current].children.insert(ch, next);
                        next
                    }
                };
            }

            // Set the bit for this pattern in the output mask
            self.nodes[current].output |= 1u64 << pattern_idx;
        }

        Ok(())
    }

    /// Builds failure links using breadth-first traversal
    fn build_failure_links(&mut self) {
        let mut queue = VecDeque::new();
        
        // Initialize root's children
        {
            let root_children: Vec<_> = self.nodes[self.root].children.values().copied().collect();
            for &child in &root_children {
                self.nodes[child].failure = Some(self.root);
                queue.push_back(child);
            }
        }

        // Process remaining nodes breadth-first
        while let Some(current) = queue.pop_front() {
            // Collect all data we need before any mutable borrows
            let current_failure = self.nodes[current].failure.unwrap_or(self.root);
            let children: Vec<_> = self.nodes[current].children.iter()
                .map(|(&ch, &node)| (ch, node))
                .collect();
            
            for (ch, child) in children {
                queue.push_back(child);
                
                // Find failure link by following parent's failure links
                let mut failure = current_failure;
                let mut next_failure = self.root;
                
                // Find the deepest node labeled by proper suffix
                while failure != self.root {
                    if let Some(&next) = self.nodes[failure].children.get(&ch) {
                        next_failure = next;
                        break;
                    }
                    failure = self.nodes[failure].failure.unwrap_or(self.root);
                }
                
                // Check root's children if we haven't found a match
                if failure == self.root {
                    next_failure = self.nodes[self.root].children.get(&ch).copied().unwrap_or(self.root);
                }

                // Set failure link and merge output masks
                let output_mask = self.nodes[next_failure].output;
                let child_node = &mut self.nodes[child];
                child_node.failure = Some(next_failure);
                child_node.output |= output_mask;
            }
        }
    }

    /// Finds the next state using goto and failure functions
    fn find_next_state(&self, mut current: usize, ch: char) -> usize {
        while !self.nodes[current].children.contains_key(&ch) && current != self.root {
            current = self.nodes[current].failure.unwrap_or(self.root);
        }
        self.nodes[current].children.get(&ch).copied().unwrap_or(self.root)
    }

    /// Helper function to check if a position is at a word boundary
    fn is_word_boundary(&self, text: &str, start: usize, end: usize) -> bool {
        if !self.config.word_boundaries {
            return true;
        }

        let is_boundary_char = |c: char| !c.is_alphanumeric();
        let before_is_boundary = start == 0 || text[..start].chars().next_back().map_or(true, is_boundary_char);
        let after_is_boundary = end >= text.len() || text[end..].chars().next().map_or(true, is_boundary_char);
        
        before_is_boundary && after_is_boundary
    }

    /// Finds all occurrences of any pattern in the given text.
    pub fn find_all<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Match> + 'a {
        let mut matches = Vec::new();
        let mut current = self.root;

        // Convert text to chars once and store with positions
        let chars: Vec<(usize, char)> = text.char_indices().collect();
        
        // Track matches at each position if we need longest-match-only
        let mut matches_at_pos = if self.config.longest_match_only {
            vec![Vec::new(); chars.len()]
        } else {
            Vec::new()
        };
        
        for (pos, (char_pos, ch)) in chars.iter().enumerate() {
            current = self.find_next_state(current, *ch);

            // Check for matches at current state
            if self.nodes[current].output != 0 {
                // Find all matching patterns at this position
                for pattern_idx in 0..self.patterns.len() {
                    if self.nodes[current].output & (1u64 << pattern_idx) != 0 {
                        let pattern_len = self.patterns[pattern_idx].chars().count();
                        if pos >= pattern_len - 1 {
                            let start_pos = pos - (pattern_len - 1);
                            let start_byte = chars[start_pos].0;
                            let end_byte = *char_pos + ch.len_utf8();
                            
                            // Check word boundaries if needed
                            if self.is_word_boundary(text, start_byte, end_byte) {
                                let m = Match {
                                    pattern_index: pattern_idx,
                                    start: start_byte,
                                    end: end_byte,
                                };
                                
                                if self.config.longest_match_only {
                                    matches_at_pos[start_pos].push(m);
                                } else {
                                    matches.push(m);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process matches if using longest-match-only
        if self.config.longest_match_only {
            matches = matches_at_pos.into_iter()
                .filter(|pos_matches| !pos_matches.is_empty())
                .map(|mut pos_matches| {
                    pos_matches.sort_by_key(|m| (std::cmp::Reverse(m.end - m.start), m.pattern_index));
                    pos_matches[0].clone()
                })
                .collect();
        }

        matches.into_iter()
    }

    /// Finds the first occurrence of any pattern in the given text.
    pub fn find_first(&self, text: &str) -> Option<Match> {
        self.find_all(text).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_patterns() {
        let result = AhoCorasick::new(vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_pattern() {
        let result = AhoCorasick::new(vec!["".to_string()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_single_pattern() {
        let ac = AhoCorasick::new(vec!["test".to_string()]).unwrap();
        let matches: Vec<_> = ac.find_all("this is a test case").collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_index, 0);
        assert_eq!(matches[0].start, 10);
        assert_eq!(matches[0].end, 14);
    }

    #[test]
    fn test_multiple_patterns() {
        let patterns: Vec<String> = vec!["he", "she", "his", "hers"]
            .into_iter()
            .map(String::from)
            .collect();
        
        // Test with default config (all matches)
        let ac = AhoCorasick::new(patterns.clone()).unwrap();
        let matches: Vec<_> = ac.find_all("she sells seashells").collect();
        assert_eq!(matches.len(), 4); // "he" in "she", "he" in "sells", "he" in "seashells"
        
        // Test with word boundaries
        let ac = AhoCorasick::with_config(patterns.clone(), MatchConfig {
            word_boundaries: true,
            longest_match_only: false,
        }).unwrap();
        let matches: Vec<_> = ac.find_all("she sells seashells").collect();
        assert_eq!(matches.len(), 1); // only "she"
        assert_eq!(matches[0].pattern_index, 1);
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, 3);
    }

    #[test]
    fn test_overlapping_patterns() {
        let patterns: Vec<String> = vec!["ant", "ant colony", "colony"]
            .into_iter()
            .map(String::from)
            .collect();
        
        // Test with default config (all matches)
        let ac = AhoCorasick::new(patterns.clone()).unwrap();
        let matches: Vec<_> = ac.find_all("ant colony").collect();
        assert_eq!(matches.len(), 3);
        
        // Test with longest-match-only
        let ac = AhoCorasick::with_config(patterns, MatchConfig {
            word_boundaries: false,
            longest_match_only: true,
        }).unwrap();
        let matches: Vec<_> = ac.find_all("ant colony").collect();
        assert_eq!(matches.len(), 2); // "ant colony" and "colony"
    }

    #[test]
    fn test_unicode() {
        let patterns: Vec<String> = vec!["🦀", "🦀🔧", "🔧"]
            .into_iter()
            .map(String::from)
            .collect();
        
        // Test with default config (all matches)
        let ac = AhoCorasick::new(patterns.clone()).unwrap();
        let matches: Vec<_> = ac.find_all("🦀🔧").collect();
        assert_eq!(matches.len(), 3);
        
        // Test with longest-match-only
        let ac = AhoCorasick::with_config(patterns, MatchConfig {
            word_boundaries: false,
            longest_match_only: true,
        }).unwrap();
        let matches: Vec<_> = ac.find_all("🦀🔧").collect();
        assert_eq!(matches.len(), 2); // "��🔧" and "🔧"
    }
}
