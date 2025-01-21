use std::collections::HashMap;

/// A node in the suffix tree.
#[derive(Debug)]
pub struct SuffixNode {
    /// Map from character -> child node index
    children: HashMap<char, usize>,

    /// Suffix link
    suffix_link: Option<usize>,

    /// Start index of the edge label in the text
    start: i32,

    /// End index of the edge label in the text (-1 means it's a leaf using `leaf_end`)
    end: i32,

    /// For leaves, once the tree is fully built, the suffix index is set.
    suffix_index: i32,
}

/// A suffix tree for a given string, built via Ukkonen's algorithm.
pub struct SuffixTree {
    /// The text, stored as characters
    text: Vec<char>,

    /// A list of all nodes
    nodes: Vec<SuffixNode>,

    /// The index of the root in `nodes`
    root: usize,

    /// Active node index
    active_node: usize,

    /// Index of the active edge (character) in `text`
    active_edge: i32,

    /// How many characters in the current edge are matched
    active_length: i32,

    /// How many suffixes remain to be added in the current phase
    remainder: i32,

    /// Internal node from the last split (if any), awaiting a suffix link
    last_new_node: Option<usize>,

    /// Global end index for leaves (we treat any node with end == -1 as a leaf)
    leaf_end: i32,
}

impl SuffixTree {
    /// Create a new (empty) suffix tree object for the given string.
    pub fn new<S: AsRef<str>>(input: S) -> Self {
        let text: Vec<char> = input.as_ref().chars().collect();
        // Pre-allocate up to 2 * text.len(), or more, to reduce reallocation
        let capacity = 2 * text.len().max(16);

        // Create a root node
        let root_node = SuffixNode {
            children: HashMap::new(),
            suffix_link: None,
            start: -1,
            end: -1,
            suffix_index: -1,
        };

        let mut nodes = Vec::with_capacity(capacity);
        nodes.push(root_node);

        Self {
            text,
            nodes,
            root: 0,
            active_node: 0,
            active_edge: -1,
            active_length: 0,
            remainder: 0,
            last_new_node: None,
            leaf_end: -1,
        }
    }

    /// Public method to build the suffix tree with Ukkonen's algorithm.
    pub fn build(&mut self) {
        for i in 0..self.text.len() {
            self.extend(i as i32);
        }
        // Assign suffix indices (and optionally print edges).
        self.assign_suffix_indices_dfs(self.root, 0);
    }

    /// Returns how many nodes are currently in the tree
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Allocate a new node and return its index in `self.nodes`.
    fn new_node(&mut self, start: i32, end: i32) -> usize {
        let node = SuffixNode {
            children: HashMap::new(),
            // Typically for internal nodes, we link to root by default
            suffix_link: Some(self.root),
            start,
            end,
            suffix_index: -1,
        };
        self.nodes.push(node);
        self.nodes.len() - 1
    }

    /// Returns the effective edge length of a node: `node.end - node.start + 1`
    /// If `node.end == -1`, we treat it as a leaf using `self.leaf_end`.
    fn edge_length(&self, node_idx: usize) -> i32 {
        let node = &self.nodes[node_idx];
        if node.start == -1 {
            return 0; // root
        }
        let end = if node.end == -1 {
            self.leaf_end
        } else {
            node.end
        };
        end - node.start + 1
    }

    /// "Walk down" to a child node if `active_length` >= edge_length(child).
    /// Returns true if we walked down, false otherwise.
    fn walk_down(&mut self, next_node: usize) -> bool {
        let edge_len = self.edge_length(next_node);
        if self.active_length >= edge_len {
            self.active_edge += edge_len;
            self.active_length -= edge_len;
            self.active_node = next_node;
            true
        } else {
            false
        }
    }

    /// Extend the suffix tree by adding the character at `pos` in `self.text`.
    fn extend(&mut self, pos: i32) {
        // We are adding a new character that extends all leaves to position `pos`
        self.leaf_end = pos;
        self.remainder += 1;
        self.last_new_node = None;

        while self.remainder > 0 {
            if self.active_length == 0 {
                self.active_edge = pos;
            }

            let active_char = self.text[self.active_edge as usize];

            // We do lookups in a narrower scope so we don't keep a long-lived mutable ref
            if !self.nodes[self.active_node].children.contains_key(&active_char) {
                // No edge with `active_char`: create a new leaf node
                let leaf_idx = self.new_node(pos, -1);
                // Insert in a small block, so this mutable borrow ends quickly
                {
                    let active_node_ref = &mut self.nodes[self.active_node];
                    active_node_ref.children.insert(active_char, leaf_idx);
                }

                // If there was an internal node from a previous extension, link it to current active_node
                if let Some(internal_idx) = self.last_new_node {
                    self.nodes[internal_idx].suffix_link = Some(self.active_node);
                    self.last_new_node = None;
                }
            } else {
                // Edge exists. We'll either walk down or split.
                let next_node_idx = *self.nodes[self.active_node]
                    .children
                    .get(&active_char)
                    .unwrap();

                if self.walk_down(next_node_idx) {
                    continue;
                }

                // If the next character on the edge is the same as the new char, just extend.
                let next_start = self.nodes[next_node_idx].start;
                let next_char_on_edge =
                    self.text[(next_start + self.active_length) as usize];

                if next_char_on_edge == self.text[pos as usize] {
                    // If an internal node was waiting for a suffix link, link it to active_node
                    if let Some(internal_idx) = self.last_new_node {
                        self.nodes[internal_idx].suffix_link = Some(self.active_node);
                        self.last_new_node = None;
                    }
                    self.active_length += 1;
                    break;
                }

                // We need to split the edge.
                let split_start = next_start;
                let split_end = split_start + self.active_length - 1;
                let split_node_idx = self.new_node(split_start, split_end);

                // Insert the split node as child of the active_node
                {
                    let active_node_ref = &mut self.nodes[self.active_node];
                    active_node_ref.children.insert(active_char, split_node_idx);
                }

                // Create a leaf node for the newly added character
                let leaf_idx = self.new_node(pos, -1);
                {
                    let split_node_ref = &mut self.nodes[split_node_idx];
                    split_node_ref
                        .children
                        .insert(self.text[pos as usize], leaf_idx);
                }

                // Update the original next_node to start after the split
                {
                    let next_node_ref = &mut self.nodes[next_node_idx];
                    next_node_ref.start += self.active_length;
                }

                let splitted_char = self.text[self.nodes[next_node_idx].start as usize];
                // Link the old node as a child of the new split node
                {
                    let split_node_ref = &mut self.nodes[split_node_idx];
                    split_node_ref.children.insert(splitted_char, next_node_idx);
                }

                // If we had an internal node from the last extension waiting for suffix link, connect it
                if let Some(internal_idx) = self.last_new_node {
                    self.nodes[internal_idx].suffix_link = Some(split_node_idx);
                }
                self.last_new_node = Some(split_node_idx);
            }

            self.remainder -= 1;

            // Move active point if necessary
            if self.active_node == self.root && self.active_length > 0 {
                self.active_length -= 1;
                self.active_edge = pos - self.remainder + 1;
            } else if self.active_node != self.root {
                let link = self.nodes[self.active_node]
                    .suffix_link
                    .unwrap_or(self.root);
                self.active_node = link;
            }
        }
    }

    /// DFS to assign suffix indices to leaves, and optionally print edges.
    fn assign_suffix_indices_dfs(&mut self, node_idx: usize, depth: i32) {
        // Copy out node.start/end so we don't hold the borrow
        let (start, end) = {
            let node = &self.nodes[node_idx];
            let e = if node.end == -1 { self.leaf_end } else { node.end };
            (node.start, e)
        };

        let mut is_leaf = true;

        // Collect children in a separate vector so we do not keep borrowing self.nodes
        let children: Vec<(char, usize)> =
            self.nodes[node_idx].children.iter().map(|(c, &i)| (*c, i)).collect();

        for (_, child_idx) in children {
            is_leaf = false;
            let edge_len = self.edge_length(child_idx);
            self.assign_suffix_indices_dfs(child_idx, depth + edge_len);
        }

        if is_leaf {
            // A leaf => suffix_index = total_length - depth
            let total_len = self.text.len() as i32;
            self.nodes[node_idx].suffix_index = total_len - depth;
        }

        // Example: print the edge label from `start..=end`
        if start != -1 {
            print!("Edge label: ");
            let last = end.min(self.text.len() as i32 - 1);
            for i in start..=last {
                print!("{}", self.text[i as usize]);
            }
            if is_leaf {
                print!("  [leaf suffix_index = {}]", self.nodes[node_idx].suffix_index);
            }
            println!();
        }
    }
}

// Example usage/test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_suffix_tree_build() {
        let mut st = SuffixTree::new("xabxa#babxba$");
        st.build();

        // Check we have more than 1 node
        assert!(st.node_count() > 1);
        // You can add more specific correctness checks here.
    }
}
