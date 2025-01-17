use std::collections::{HashMap, VecDeque, HashSet};

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
    /// Dictionary patterns ending at this node
    patterns: Vec<usize>,
    /// Depth in trie (for match position calculation)
    depth: usize,
}

impl TrieNode {
    fn new(depth: usize) -> Self {
        Self {
            children: HashMap::new(),
            failure: None,
            patterns: Vec::new(),
            depth,
        }
    }
}

/// An implementation of the Aho-Corasick string matching algorithm.
/// 
/// This algorithm efficiently finds multiple patterns in a text simultaneously by constructing
/// a finite state machine from the patterns and scanning the text in a single pass.
/// 
/// # Example
/// ```
/// use blocks_cs_string::algorithms::AhoCorasick;
/// 
/// let patterns = vec!["he", "she", "his", "hers"];
/// let ac = AhoCorasick::new(patterns).unwrap();
/// let text = "she sells seashells";
/// let matches: Vec<_> = ac.find_all(text).collect();
/// assert_eq!(matches.len(), 1);
/// ```
#[derive(Debug)]
pub struct AhoCorasick {
    /// All nodes in the automaton
    nodes: Vec<TrieNode>,
    /// Original patterns for reporting matches
    patterns: Vec<String>,
    /// Root node is always at index 0
    root: usize,
}

impl AhoCorasick {
    /// Creates a new Aho-Corasick automaton from the given patterns.
    /// 
    /// # Arguments
    /// * `patterns` - A vector of strings to search for
    /// 
    /// # Returns
    /// * `Ok(AhoCorasick)` - Successfully constructed automaton
    /// * `Err(String)` - Error message if construction fails
    /// 
    /// # Example
    /// ```
    /// use blocks_cs_string::algorithms::AhoCorasick;
    /// 
    /// let patterns = vec!["cat", "dog", "rat"];
    /// let ac = AhoCorasick::new(patterns).unwrap();
    /// ```
    pub fn new(patterns: Vec<String>) -> Result<Self, String> {
        if patterns.is_empty() {
            return Err("At least one pattern is required".to_string());
        }

        let mut ac = Self {
            nodes: vec![TrieNode::new(0)],
            patterns,
            root: 0,
        };

        // Build trie
        ac.build_trie()?;
        
        // Build failure links
        ac.build_failure_links();

        Ok(ac)
    }

    /// Builds the initial trie structure from the patterns
    fn build_trie(&mut self) -> Result<(), String> {
        for (pattern_idx, pattern) in self.patterns.iter().enumerate() {
            if pattern.is_empty() {
                return Err("Empty patterns are not allowed".to_string());
            }

            let mut current = self.root;
            let mut depth = 0;

            for ch in pattern.chars() {
                depth += 1;
                current = match self.nodes[current].children.get(&ch) {
                    Some(&next) => next,
                    None => {
                        let next = self.nodes.len();
                        self.nodes.push(TrieNode::new(depth));
                        self.nodes[current].children.insert(ch, next);
                        next
                    }
                };
            }

            self.nodes[current].patterns.push(pattern_idx);
        }
        Ok(())
    }

    /// Builds failure links using breadth-first traversal
    fn build_failure_links(&mut self) {
        let mut queue = VecDeque::new();
        
        // Collect root's children first
        let root_children: Vec<_> = self.nodes[self.root]
            .children
            .values()
            .copied()
            .collect();
            
        // Set root's children failure links to root
        for child in root_children {
            self.nodes[child].failure = Some(self.root);
            queue.push_back(child);
        }

        // Process remaining nodes
        while let Some(current) = queue.pop_front() {
            // Collect children and their transitions first
            let children: Vec<(char, usize)> = self.nodes[current]
                .children
                .iter()
                .map(|(&ch, &node)| (ch, node))
                .collect();
                
            for (ch, child) in children {
                queue.push_back(child);

                let mut failure = self.nodes[current].failure.unwrap_or(self.root);
                
                loop {
                    if let Some(&next) = self.nodes[failure].children.get(&ch) {
                        self.nodes[child].failure = Some(next);
                        break;
                    }
                    if failure == self.root {
                        self.nodes[child].failure = Some(self.root);
                        break;
                    }
                    failure = self.nodes[failure].failure.unwrap_or(self.root);
                }
            }
        }
    }

    /// Finds all occurrences of any pattern in the given text.
    /// 
    /// Returns an iterator over all matches found in the text.
    /// 
    /// # Arguments
    /// * `text` - The text to search in
    /// 
    /// # Example
    /// ```
    /// use blocks_cs_string::algorithms::AhoCorasick;
    /// 
    /// let patterns = vec!["he", "she", "his", "hers"];
    /// let ac = AhoCorasick::new(patterns).unwrap();
    /// let text = "she sells seashells";
    /// let matches: Vec<_> = ac.find_all(text).collect();
    /// ```
    pub fn find_all<'a>(&'a self, text: &'a str) -> impl Iterator<Item = Match> + 'a {
        let mut matches = Vec::new();
        let mut current = self.root;

        for (pos, ch) in text.chars().enumerate() {
            loop {
                if let Some(&next) = self.nodes[current].children.get(&ch) {
                    current = next;
                    break;
                }
                if current == self.root {
                    break;
                }
                current = self.nodes[current].failure.unwrap_or(self.root);
            }

            // Add matches for current node and all its suffix links
            let mut state = current;
            while state != self.root {
                for &pattern_idx in &self.nodes[state].patterns {
                    let pattern_len = self.patterns[pattern_idx].chars().count();
                    matches.push(Match {
                        pattern_index: pattern_idx,
                        start: pos + 1 - pattern_len,
                        end: pos + 1,
                    });
                }
                state = self.nodes[state].failure.unwrap_or(self.root);
            }
        }

        matches.into_iter()
    }

    /// Finds the first occurrence of any pattern in the given text.
    /// 
    /// # Arguments
    /// * `text` - The text to search in
    /// 
    /// # Returns
    /// * `Some(Match)` - First match found
    /// * `None` - No matches found
    /// 
    /// # Example
    /// ```
    /// use blocks_cs_string::algorithms::AhoCorasick;
    /// 
    /// let patterns = vec!["he", "she", "his", "hers"];
    /// let ac = AhoCorasick::new(patterns).unwrap();
    /// let text = "she sells seashells";
    /// if let Some(m) = ac.find_first(text) {
    ///     println!("Found pattern {} at position {}", m.pattern_index, m.start);
    /// }
    /// ```
    pub fn find_first(&self, text: &str) -> Option<Match> {
        self.find_all(text).next()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_matches() {
        let patterns = vec!["he", "she", "his", "hers"]
            .into_iter()
            .map(String::from)
            .collect();
        let ac = AhoCorasick::new(patterns).unwrap();
        let matches: Vec<_> = ac.find_all("she sells").collect();
        println!("Debug matches: {:?}", matches);
        for m in &matches {
            println!("Match: pattern_index={}, start={}, end={}", 
                m.pattern_index, m.start, m.end);
        }
    }

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
        let patterns = vec!["he", "she", "his", "hers"]
            .into_iter()
            .map(String::from)
            .collect();
        let ac = AhoCorasick::new(patterns).unwrap();
        let matches: Vec<_> = ac.find_all("she sells seashells").collect();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].pattern_index, 1); // "she"
        assert_eq!(matches[0].start, 0);
        assert_eq!(matches[0].end, 3);
    }

    #[test]
    fn test_overlapping_patterns() {
        let patterns = vec!["ant", "ant colony", "colony"]
            .into_iter()
            .map(String::from)
            .collect();
        let ac = AhoCorasick::new(patterns).unwrap();
        let matches: Vec<_> = ac.find_all("ant colony").collect();
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_unicode() {
        let patterns = vec!["🦀", "🦀🔧", "🔧"]
            .into_iter()
            .map(String::from)
            .collect();
        let ac = AhoCorasick::new(patterns).unwrap();
        let matches: Vec<_> = ac.find_all("🦀🔧").collect();
        assert_eq!(matches.len(), 3);
    }
}
