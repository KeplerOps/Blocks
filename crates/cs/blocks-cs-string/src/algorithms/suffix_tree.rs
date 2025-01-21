use std::collections::{HashMap, BTreeMap};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Represents the Suffix Tree.
///
/// Fields:
/// - `text`: the input text as a vector of characters
/// - `root`: the index of the root node
/// - `nodes`: the array of nodes in the tree
/// - `active_node`: the current active node for Ukkonen's algorithm
/// - `active_edge`: the current active edge character
/// - `active_length`: the length of the active point on the current edge
/// - `remaining_suffix_count`: the number of suffixes yet to be added
/// - `end`: a shared pointer to the current end index
/// - `parent_map`: a map to keep track of parent nodes during construction
#[derive(Debug)]
pub struct SuffixTree {
    text: Vec<char>,
    root: usize,
    nodes: Vec<Node>,
    active_node: usize,
    active_edge: Option<char>,
    active_length: usize,
    remaining_suffix_count: usize,
    end: Arc<AtomicUsize>,
    parent_map: HashMap<usize, usize>,
}

/// A node in the Suffix Tree.
///
/// - `children`: maps from character to `Edge`.
/// - `suffix_link`: for Ukkonen's algorithm (points to another node).
#[derive(Debug, Default)]
pub struct Node {
    children: BTreeMap<char, Edge>,
    suffix_link: Option<usize>,
}

impl Node {
    fn new() -> Self {
        Self {
            children: BTreeMap::<char, Edge>::new(),
            suffix_link: None,
        }
    }
}

/// Represents an edge in the tree.
///
/// Instead of storing the entire substring, we store:
/// - `start` and `end` indices into `SuffixTree::text`
/// - `target_node`: the index of the node this edge leads to
#[derive(Debug, Clone)]
struct Edge {
    start: usize,
    end: EdgeEnd,
    target_node: usize,
}

impl Edge {
    fn new(start: usize, end: EdgeEnd, target_node: usize) -> Self {
        Self {
            start,
            end,
            target_node,
        }
    }
}

/// Either a specific index in the text or a pointer to the global end.
#[derive(Debug, Clone)]
enum EdgeEnd {
    Fixed(usize),
    Shared(Arc<AtomicUsize>),
}

impl EdgeEnd {
    fn value(&self) -> usize {
        match self {
            Self::Fixed(v) => *v,
            Self::Shared(v) => v.load(Ordering::SeqCst),
        }
    }

    fn is_valid(&self) -> bool {
        match self {
            Self::Fixed(_) => true,
            Self::Shared(v) => v.load(Ordering::SeqCst) != usize::MAX,
        }
    }
}

impl SuffixTree {
    /// Create a new suffix tree from the given text.
    /// Appends a sentinel character `$` to ensure uniqueness of suffix ends.
    pub fn new(text: &str) -> Self {
        let mut chars: Vec<char> = text.chars().collect();
        if !chars.ends_with(&['$']) {
            chars.push('$');
        }

        let mut tree = Self {
            text: chars,
            root: 0,
            nodes: vec![Node::new()],
            active_node: 0,
            active_edge: None,
            active_length: 0,
            remaining_suffix_count: 0,
            end: Arc::new(AtomicUsize::new(0)),
            parent_map: HashMap::<usize, usize>::new(),
        };

        for i in 0..tree.text.len() {
            tree.extend_suffix_tree(i);
        }

        tree
    }

    /// Extends the suffix tree with a new character at position i.
    fn extend_suffix_tree(&mut self, i: usize) {
        self.end.store(i, Ordering::SeqCst);
        self.remaining_suffix_count += 1;
        let mut last_new_node: Option<usize> = None;

        while self.remaining_suffix_count > 0 {
            if self.active_length == 0 {
                self.active_edge = Some(self.text[i]);
            }

            let active_char = self.active_edge.unwrap();
            if !self.nodes[self.active_node].children.contains_key(&active_char) {
                // No edge, create a new leaf edge
                let leaf_node_idx = self.new_node();
                let edge = Edge::new(
                    if i >= self.remaining_suffix_count {
                        i - self.remaining_suffix_count + 1
                    } else {
                        0
                    },
                    EdgeEnd::Shared(self.end.clone()),
                    leaf_node_idx,
                );
                self.nodes[self.active_node].children.insert(active_char, edge);
                self.parent_map.insert(leaf_node_idx, self.active_node);

                // Add suffix link if needed
                if let Some(last_idx) = last_new_node {
                    self.nodes[last_idx].suffix_link = Some(self.active_node);
                }
                last_new_node = None;
            } else {
                // Edge exists
                let edge = self.nodes[self.active_node].children[&active_char].clone();
                let next_char = self.text[edge.start + self.active_length];

                if next_char == self.text[i] {
                    // Character already exists on edge
                    self.active_length += 1;
                    let edge_len = self.edge_length(&edge);

                    if self.active_length >= edge_len {
                        self.active_node = edge.target_node;
                        self.active_length = 0;
                        self.active_edge = None;
                    }
                    break;
                }

                // Split edge
                let split_node_idx = self.new_node();
                let old_edge = self.nodes[self.active_node].children.remove(&active_char).unwrap();
                
                // Create new internal node
                let new_internal_edge = Edge::new(
                    old_edge.start,
                    EdgeEnd::Fixed(old_edge.start + self.active_length - 1),
                    split_node_idx,
                );
                self.nodes[self.active_node].children.insert(active_char, new_internal_edge);
                self.parent_map.insert(split_node_idx, self.active_node);

                // Create new leaf node
                let leaf_node_idx = self.new_node();
                let new_leaf_edge = Edge::new(
                    i,
                    EdgeEnd::Shared(self.end.clone()),
                    leaf_node_idx,
                );
                self.nodes[split_node_idx].children.insert(self.text[i], new_leaf_edge);
                self.parent_map.insert(leaf_node_idx, split_node_idx);

                // Adjust old edge
                let adjusted_edge = Edge::new(
                    old_edge.start + self.active_length,
                    old_edge.end,
                    old_edge.target_node,
                );
                self.nodes[split_node_idx].children.insert(next_char, adjusted_edge);
                self.parent_map.insert(old_edge.target_node, split_node_idx);

                // Add suffix link if needed
                if let Some(last_idx) = last_new_node {
                    self.nodes[last_idx].suffix_link = Some(split_node_idx);
                }
                last_new_node = Some(split_node_idx);
            }

            self.remaining_suffix_count -= 1;

            if self.active_node == self.root && self.active_length > 0 {
                self.active_length -= 1;
                self.active_edge = Some(self.text[i - self.remaining_suffix_count + 1]);
            } else if self.active_node != self.root {
                self.active_node = self.nodes[self.active_node].suffix_link.unwrap_or(self.root);
            }
        }
    }

    /// Return the length of the current edge, which depends on the global end if it's shared.
    fn edge_length(&self, edge: &Edge) -> usize {
        match &edge.end {
            EdgeEnd::Fixed(end) => end - edge.start + 1,
            EdgeEnd::Shared(e) => {
                let end = e.load(Ordering::SeqCst);
                if end == usize::MAX {
                    0 // Special case for initial state
                } else {
                    end - edge.start + 1
                }
            }
        }
    }

    /// Create a new node and push it to the `nodes` array. Returns its index.
    fn new_node(&mut self) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(Node::new());
        idx
    }

    // --- Optional utility functions for demonstration ---

    /// Check if a substring exists in the suffix tree (simple lookup).
    /// Returns `true` if found, `false` otherwise.
    pub fn contains(&self, pattern: &str) -> bool {
        self.find_pattern_node(pattern).is_some()
    }

    /// Print the edges of the suffix tree (for debugging).
    /// Each edge is printed as (node_index -> target_node_index, substring).
    pub fn debug_print(&self) {
        for (i, node) in self.nodes.iter().enumerate() {
            for (ch, edge) in &node.children {
                let substr: String = match &edge.end {
                    EdgeEnd::Fixed(f) => {
                        self.text[edge.start..=*f].iter().collect::<String>()
                    },
                    EdgeEnd::Shared(e) => {
                        let end: usize = e.load(Ordering::SeqCst);
                        self.text[edge.start..=end].iter().collect::<String>()
                    }
                };
                println!(
                    "Node {} --({}:'{}')-> Node {}",
                    i, ch, substr, edge.target_node
                );
            }
            if let Some(suffix_link) = node.suffix_link {
                println!("  (Suffix link from {} to {})", i, suffix_link);
            }
        }
    }

    /// Find the node that corresponds to the end of the given pattern.
    /// Returns Some(node_index) if found, None if not found.
    fn find_pattern_node(&self, pattern: &str) -> Option<usize> {
        let mut current_node: usize = self.root;
        let mut chars = pattern.chars().peekable();

        while let Some(&c) = chars.peek() {
            if let Some(edge) = self.nodes[current_node].children.get(&c) {
                let edge_len: usize = self.edge_length(edge);
                let path_str: String = self.text[edge.start..(edge.start + edge_len)]
                    .iter()
                    .collect::<String>();

                // Walk along edge
                let mut path_chars = path_str.chars();
                while let Some(pattern_char) = chars.next() {
                    match path_chars.next() {
                        Some(edge_char) if edge_char == pattern_char => continue,
                        _ => return None,
                    }
                }

                // If we've consumed all pattern characters but still have edge characters,
                // that's okay - we've found a match
                current_node = edge.target_node;
            } else {
                return None;
            }
        }
        Some(current_node)
    }

    /// Find all occurrences of a pattern in the text.
    /// Returns a vector of starting indices where the pattern occurs.
    pub fn find_all(&self, pattern: &str) -> Vec<usize> {
        if let Some(node) = self.find_pattern_node(pattern) {
            self.collect_leaf_positions(node)
        } else {
            Vec::new()
        }
    }

    /// Helper method to collect all leaf positions under a node
    fn collect_leaf_positions(&self, node: usize) -> Vec<usize> {
        let mut positions: Vec<usize> = Vec::new();
        self.collect_leaf_positions_helper(node, &mut positions);
        positions.sort_unstable();
        positions
    }

    fn collect_leaf_positions_helper(&self, node: usize, positions: &mut Vec<usize>) {
        if self.nodes[node].children.is_empty() {
            // This is a leaf - we need to find its starting position
            let mut current: usize = node;
            let mut length: usize = 0;

            // Walk up to root to calculate the total length
            while current != self.root {
                // Find the edge that leads to current node
                for (_, edge) in &self.nodes[self.parent_map[&current]].children {
                    if edge.target_node == current {
                        length += self.edge_length(edge);
                        break;
                    }
                }
                current = self.parent_map[&current];
            }

            // The starting position is text.len() - length
            positions.push(self.text.len() - length);
        } else {
            // Recursively visit all children
            for edge in self.nodes[node].children.values() {
                self.collect_leaf_positions_helper(edge.target_node, positions);
            }
        }
    }

    /// Get all suffixes in the text.
    /// Returns a vector of strings representing all suffixes.
    pub fn get_suffixes(&self) -> Vec<String> {
        let mut suffixes: Vec<String> = Vec::new();
        self.collect_suffixes(self.root, String::new(), &mut suffixes);
        suffixes.sort();
        suffixes
    }

    fn collect_suffixes(&self, node: usize, mut current: String, suffixes: &mut Vec<String>) {
        if self.nodes[node].children.is_empty() {
            // Add sentinel if not present
            if !current.ends_with('$') {
                current.push('$');
            }
            suffixes.push(current);
            return;
        }

        for (_, edge) in self.nodes[node].children.iter() {
            let edge_len: usize = self.edge_length(edge);
            let substr: String = self.text[edge.start..edge.start + edge_len]
                .iter()
                .collect();
            let mut new_current: String = current.clone();
            new_current.push_str(&substr);
            self.collect_suffixes(edge.target_node, new_current, suffixes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suffix_tree_contains() {
        let s = "banana";
        let tree = SuffixTree::new(s);

        assert!(tree.contains("banana"));
        assert!(tree.contains("anana"));
        assert!(tree.contains("nana"));
        assert!(tree.contains("ana"));
        assert!(tree.contains("na"));
        assert!(tree.contains("a"));
        assert!(!tree.contains("bnana"));
        assert!(!tree.contains("band"));
    }

    #[test]
    fn test_debug_print() {
        let tree = SuffixTree::new("abc");
        // Just ensure it doesn't panic
        tree.debug_print();
    }

    #[test]
    fn test_find_all() {
        let tree = SuffixTree::new("banana");
        assert_eq!(tree.find_all("ana"), vec![1, 3]);
        assert_eq!(tree.find_all("an"), vec![1, 3]);
        assert_eq!(tree.find_all("banana"), vec![0]);
        assert_eq!(tree.find_all("xyz"), vec![]);
    }

    #[test]
    fn test_get_suffixes() {
        let tree = SuffixTree::new("abc");
        let mut suffixes = tree.get_suffixes();
        suffixes.sort();
        assert_eq!(
            suffixes,
            vec![
                "abc$".to_string(),
                "bc$".to_string(),
                "c$".to_string(),
                "$".to_string()
            ]
        );
    }
}
