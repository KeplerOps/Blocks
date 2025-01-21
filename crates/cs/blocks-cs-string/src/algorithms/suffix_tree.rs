use std::collections::HashMap;

/// A node in the suffix tree
#[derive(Debug)]
struct Node {
    /// Start index in the text
    start: usize,
    /// End index in the text (exclusive)
    /// Uses Option<usize> to handle leaf nodes that extend to end of text
    end: Option<usize>,
    /// Children nodes indexed by their first character
    children: HashMap<char, usize>,
    /// Suffix link to another node (used in Ukkonen's algorithm)
    suffix_link: Option<usize>,
}

impl Node {
    fn new(start: usize, end: Option<usize>) -> Self {
        Self {
            start,
            end,
            children: HashMap::new(),
            suffix_link: None,
        }
    }

    /// Get the character position for a byte position
    fn get_char_pos(&self, text: &str, byte_pos: usize) -> usize {
        text.chars().take(byte_pos).count()
    }

    /// Returns the length of the edge label
    fn edge_length(&self, current_end: usize) -> usize {
        self.end.unwrap_or(current_end) - self.start
    }
}

/// Active point structure used in Ukkonen's algorithm
#[derive(Debug, Clone)]
struct ActivePoint {
    /// Node number of active node
    node: usize,
    /// Character offset into the active edge
    length: usize,
    /// First character of active edge (None if no active edge)
    edge: Option<char>,
}

/// A suffix tree implementation using Ukkonen's algorithm.
/// 
/// Provides O(n) construction time and space complexity, where n is the length of the text.
/// Supports efficient pattern matching and other string operations.
#[derive(Debug)]
pub struct SuffixTree {
    /// The text being indexed
    text: String,
    /// Vector of nodes
    nodes: Vec<Node>,
    /// Current end value for leaves
    end: usize,
    /// Root node is always at index 0
    root: usize,
    /// Tracks the last created internal node for suffix link creation
    last_internal_node: Option<usize>,
    /// Active point for Ukkonen's algorithm
    active: ActivePoint,
    /// Number of suffixes yet to be inserted
    remainder: usize,
}

impl SuffixTree {
    /// Constructs a new suffix tree from the given text using Ukkonen's algorithm.
    pub fn new(text: &str) -> Self {
        let mut tree = Self {
            text: text.to_string(),
            nodes: vec![Node::new(0, Some(0))], // Root node
            end: 0,
            root: 0,
            last_internal_node: None,
            active: ActivePoint {
                node: 0,
                length: 0,
                edge: None,
            },
            remainder: 0,
        };

        // Build tree using Ukkonen's algorithm
        tree.build();
        tree
    }

    /// Builds the suffix tree using Ukkonen's algorithm
    fn build(&mut self) {
        let text_chars: Vec<char> = self.text.chars().collect();
        
        // Phase i handles prefix text[0..i]
        for (i, &ch) in text_chars.iter().enumerate() {
            self.end = i + 1;
            self.last_internal_node = None;
            self.remainder += 1;

            // Extension j handles suffix text[j..i]
            while self.remainder > 0 {
                if self.active.length == 0 {
                    // Explicit node: try to add edge directly
                    if let Some(&edge) = self.nodes[self.active.node].children.get(&ch) {
                        // Already exists, advance active point
                        self.active.edge = Some(ch);
                        self.active.length = 1;
                        break; // Rule 3
                    } else {
                        // Create new leaf
                        let leaf = self.nodes.len();
                        self.nodes.push(Node::new(i, None));
                        self.nodes[self.active.node].children.insert(ch, leaf);
                        self.remainder -= 1;
                    }
                } else {
                    // Implicit node: check next character
                    if !self.test_and_split(i, ch) {
                        break; // Rule 3
                    }
                    self.remainder -= 1;
                }

                // Follow suffix link or move to root
                if self.active.node == self.root && self.active.length > 0 {
                    self.active.length -= 1;
                    self.active.edge = text_chars.get(i - self.remainder + 1).copied();
                } else {
                    self.active.node = self.nodes[self.active.node].suffix_link.unwrap_or(self.root);
                }
            }
        }
    }

    /// Tests if current phase/extension is already handled and splits if necessary
    /// Returns true if extension is needed
    fn test_and_split(&mut self, pos: usize, ch: char) -> bool {
        let text_chars: Vec<char> = self.text.chars().collect();
        let edge_first_char = self.active.edge.unwrap();
        let edge = self.nodes[self.active.node].children[&edge_first_char];
        let edge_pos = self.nodes[edge].start + self.active.length;
        
        if text_chars[edge_pos] == ch {
            return false; // Rule 3: already exists
        }

        // Split needed
        let split = self.nodes.len();
        self.nodes.push(Node::new(
            self.nodes[edge].start,
            Some(self.nodes[edge].start + self.active.length)
        ));
        self.nodes[self.active.node].children.insert(edge_first_char, split);
        
        // Update original node
        self.nodes[edge].start += self.active.length;
        self.nodes[split].children.insert(text_chars[edge_pos], edge);
        
        // Create new leaf
        let leaf = self.nodes.len();
        self.nodes.push(Node::new(pos, None));
        self.nodes[split].children.insert(ch, leaf);

        // Handle suffix links
        if let Some(last) = self.last_internal_node {
            self.nodes[last].suffix_link = Some(split);
        }
        self.last_internal_node = Some(split);
        
        true
    }

    /// Checks if a pattern exists in the text
    pub fn contains(&self, pattern: &str) -> bool {
        if pattern.is_empty() {
            return true;
        }

        let mut node = self.root;
        let mut pos = 0;
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = self.text.chars().collect();

        'outer: while pos < pattern.len() {
            let ch = pattern_chars[pos];
            
            if let Some(&edge) = self.nodes[node].children.get(&ch) {
                let mut edge_pos = self.nodes[edge].start;
                let edge_end = self.nodes[edge].end.unwrap_or(self.end);
                
                while edge_pos < edge_end && pos < pattern.len() {
                    if text_chars[edge_pos] != pattern_chars[pos] {
                        return false;
                    }
                    edge_pos += 1;
                    pos += 1;
                }
                
                if pos < pattern.len() {
                    node = edge;
                    continue 'outer;
                }
                
                return true;
            }
            
            return false;
        }
        
        true
    }

    /// Finds all occurrences of a pattern in the text
    pub fn find_all(&self, pattern: &str) -> Vec<usize> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let mut result = Vec::new();
        let mut node = self.root;
        let mut matched = 0;
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = self.text.chars().collect();

        // First, find the node representing the pattern
        'outer: while matched < pattern.len() {
            let ch = pattern_chars[matched];
            
            if let Some(&edge) = self.nodes[node].children.get(&ch) {
                let mut edge_pos = self.nodes[edge].start;
                let edge_end = self.nodes[edge].end.unwrap_or(self.end);
                
                while edge_pos < edge_end && matched < pattern.len() {
                    if text_chars[edge_pos] != pattern_chars[matched] {
                        return result;
                    }
                    edge_pos += 1;
                    matched += 1;
                }
                
                if matched < pattern.len() {
                    node = edge;
                    continue 'outer;
                }
                
                // Pattern found, collect all positions
                self.collect_positions(edge, pattern, &mut result);
                break;
            }
            
            return result;
        }

        result.sort_unstable();
        result
    }

    /// Recursively collects starting positions from a subtree
    fn collect_positions(&self, node: usize, pattern: &str, result: &mut Vec<usize>) {
        let node_ref = &self.nodes[node];
        
        if node_ref.children.is_empty() {
            // Leaf node - calculate starting position
            let start_pos = node_ref.start;
            if start_pos >= pattern.chars().count() - 1 {
                // Convert from byte index to character index for position calculation
                let text_chars: Vec<char> = self.text.chars().collect();
                let char_pos = node_ref.get_char_pos(&self.text, start_pos);
                let pat_chars = pattern.chars().count();
                if char_pos >= pat_chars - 1 {
                    result.push(char_pos - pat_chars + 1);
                }
            }
        } else {
            // Internal node - recurse through children
            for &child in node_ref.children.values() {
                self.collect_positions(child, pattern, result);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_construction() {
        let text = "banana";
        let st = SuffixTree::new(text);

        assert!(st.contains("ana"));
        assert!(st.contains("ban"));
        assert!(st.contains("na"));
        assert!(st.contains("nan"));
    }

    #[test]
    fn test_find_all() {
        let text = "banana";
        let st = SuffixTree::new(text);

        let empty: Vec<usize> = vec![];
        assert_eq!(st.find_all("ana"), vec![1_usize, 3_usize]);
        assert_eq!(st.find_all("na"), vec![2_usize, 4_usize]);
        assert_eq!(st.find_all("a"), vec![1_usize, 3_usize, 5_usize]);
        assert_eq!(st.find_all("ban"), vec![0_usize]);
        assert_eq!(st.find_all("xyz"), empty);
    }

    #[test]
    fn test_empty_pattern() {
        let text = "banana";
        let st = SuffixTree::new(text);

        assert!(st.contains(""));
        let empty: Vec<usize> = vec![];
        assert_eq!(st.find_all(""), empty);
    }

    #[test]
    fn test_unicode_text() {
        let text = "こんにちは世界";
        let st = SuffixTree::new(text);

        assert!(st.contains("にち"));
        assert!(st.contains("世界"));
        assert!(!st.contains("世に"));

        assert_eq!(st.find_all("にち"), vec![2_usize]);
        assert_eq!(st.find_all("世界"), vec![5_usize]);
    }

    #[test]
    fn test_overlapping_patterns() {
        let text = "aaaaa";
        let st = SuffixTree::new(text);

        assert_eq!(st.find_all("aa"), vec![0_usize, 1_usize, 2_usize, 3_usize]);
        assert_eq!(st.find_all("aaa"), vec![0_usize, 1_usize, 2_usize]);
    }

    #[test]
    fn test_long_text() {
        let text = "a".repeat(1000) + "b";
        let st = SuffixTree::new(&text);

        assert!(st.contains("aaa"));
        assert!(st.contains("b"));
        assert!(!st.contains("c"));

        let positions = st.find_all("aa");
        assert_eq!(positions.len(), 999);
    }

    #[test]
    fn test_case_sensitivity() {
        let text = "bAnAnA";
        let st = SuffixTree::new(text);

        assert!(!st.contains("ana"));
        assert!(st.contains("AnA"));

        assert_eq!(st.find_all("ana"), vec![]);
        assert_eq!(st.find_all("AnA"), vec![1_usize, 3_usize]);
    }
}
