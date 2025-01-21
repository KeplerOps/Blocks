/// Suffix Automaton implementation.
/// 
/// A suffix automaton is a minimal deterministic finite automaton (DFA) that recognizes
/// all suffixes of a given string. It can be built in O(n) time and space, where n is
/// the length of the string. The automaton can be used for pattern matching, finding
/// the lexicographically minimal cyclic shift, and other string operations.
/// 
/// # Example
/// ```
/// use blocks_cs_string::algorithms::suffix_automaton::SuffixAutomaton;
/// 
/// let text = "banana";
/// let sa = SuffixAutomaton::new(text);
/// assert!(sa.contains("ana"));
/// ```

use std::collections::HashMap;

/// A state in the suffix automaton
#[derive(Debug, Clone)]
struct State {
    /// Length of the longest string in this state
    len: usize,
    /// Link to the suffix link state
    link: Option<usize>,
    /// Transitions from this state
    next: HashMap<char, usize>,
    /// First position where this state ends in the text
    first_pos: usize,
    /// Whether this state is terminal (represents a suffix)
    is_terminal: bool,
}

impl State {
    fn new(len: usize, pos: usize) -> Self {
        Self {
            len,
            link: None,
            next: HashMap::new(),
            first_pos: pos,
            is_terminal: false,
        }
    }
}

/// A suffix automaton for efficient string pattern matching
#[derive(Debug)]
pub struct SuffixAutomaton {
    /// States of the automaton
    states: Vec<State>,
    /// Last processed state
    last: usize,
    /// Original text length
    text_len: usize,
}

impl SuffixAutomaton {
    /// Creates a new suffix automaton from the given text.
    pub fn new(text: &str) -> Self {
        let mut sa = Self {
            states: vec![State::new(0, 0)],
            last: 0,
            text_len: text.chars().count(),
        };
        
        // Build the automaton by adding characters one by one
        for (i, ch) in text.chars().enumerate() {
            sa.extend(ch, i);
        }
        
        // Mark terminal states
        sa.mark_terminals();
        sa
    }

    /// Extends the automaton with a new character.
    fn extend(&mut self, ch: char, pos: usize) {
        let curr_last = self.last;
        let curr_state = self.states[curr_last].clone();
        
        // Create new state
        let new_state = self.states.len();
        self.states.push(State::new(curr_state.len + 1, pos));
        
        // Add transition from last state
        let mut p = curr_last;
        
        // Find suffix links
        while p != usize::MAX && !self.states[p].next.contains_key(&ch) {
            self.states[p].next.insert(ch, new_state);
            p = self.states[p].link.unwrap_or(usize::MAX);
        }
        
        if p == usize::MAX {
            // No matching suffix found, link to root
            self.states[new_state].link = Some(0);
        } else {
            let q = self.states[p].next[&ch];
            if self.states[p].len + 1 == self.states[q].len {
                // Direct suffix link
                self.states[new_state].link = Some(q);
            } else {
                // Clone state and create new suffix link
                let clone = self.states.len();
                let mut cloned_state = self.states[q].clone();
                cloned_state.len = self.states[p].len + 1;
                cloned_state.first_pos = self.states[q].first_pos;
                self.states.push(cloned_state);
                
                // Update transitions
                while p != usize::MAX && self.states[p].next[&ch] == q {
                    self.states[p].next.insert(ch, clone);
                    p = self.states[p].link.unwrap_or(usize::MAX);
                }
                
                self.states[q].link = Some(clone);
                self.states[new_state].link = Some(clone);
            }
        }
        
        self.last = new_state;
    }

    /// Marks terminal states in the automaton.
    fn mark_terminals(&mut self) {
        let mut state = self.last;
        while state != 0 {
            self.states[state].is_terminal = true;
            state = self.states[state].link.unwrap_or(0);
        }
        self.states[0].is_terminal = true;
    }

    /// Checks if the automaton contains the given pattern.
    pub fn contains(&self, pattern: &str) -> bool {
        if pattern.is_empty() {
            return true;
        }

        let mut state = 0;
        let pattern_len = pattern.chars().count();
        let mut curr_len = 0;
        
        for ch in pattern.chars() {
            match self.states[state].next.get(&ch) {
                Some(&next) => {
                    state = next;
                    curr_len += 1;
                    // Ensure we're following a continuous path
                    if curr_len > self.states[state].len {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Pattern must end at a state that represents a valid substring
        curr_len <= self.states[state].len
    }

    /// Finds all occurrences of a pattern in the text.
    pub fn find_all(&self, pattern: &str) -> Vec<usize> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let mut positions = Vec::new();
        let mut state = 0;
        let pattern_len = pattern.chars().count();
        let mut curr_len = 0;
        
        // First check if pattern exists in automaton
        for ch in pattern.chars() {
            match self.states[state].next.get(&ch) {
                Some(&next) => {
                    state = next;
                    curr_len += 1;
                }
                None => return positions,
            }
        }
        
        // Only proceed if we found a valid match
        if curr_len > self.states[state].len {
            return positions;
        }
        
        // Found a match, collect positions by following suffix links
        let mut curr_state = state;
        while curr_state != 0 {
            if self.states[curr_state].len >= pattern_len {
                let pos = self.states[curr_state].first_pos + 1 - pattern_len;
                if pos + pattern_len <= self.text_len {
                    positions.push(pos);
                }
            }
            curr_state = self.states[curr_state].link.unwrap_or(0);
        }
        
        positions.sort_unstable();
        positions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_construction() {
        let text = "banana";
        let sa = SuffixAutomaton::new(text);
        
        assert!(sa.contains("ana"));
        assert!(sa.contains("ban"));
        assert!(sa.contains("na"));
        assert!(!sa.contains("nan"));
    }

    #[test]
    fn test_find_all() {
        let text = "banana";
        let sa = SuffixAutomaton::new(text);
        
        assert_eq!(sa.find_all("ana"), vec![1, 3]);
        assert_eq!(sa.find_all("na"), vec![2, 4]);
        assert_eq!(sa.find_all("a"), vec![1, 3, 5]);
        assert_eq!(sa.find_all("ban"), vec![0]);
        assert_eq!(sa.find_all("xyz"), vec![]);
    }

    #[test]
    fn test_empty_pattern() {
        let text = "banana";
        let sa = SuffixAutomaton::new(text);
        
        assert!(sa.contains(""));
        assert_eq!(sa.find_all(""), vec![]);
    }

    #[test]
    fn test_unicode_text() {
        let text = "こんにちは世界";
        let sa = SuffixAutomaton::new(text);
        
        assert!(sa.contains("にち"));
        assert!(sa.contains("世界"));
        assert!(!sa.contains("世に"));
        
        assert_eq!(sa.find_all("にち"), vec![2]);
        assert_eq!(sa.find_all("世界"), vec![5]);
    }

    #[test]
    fn test_overlapping_patterns() {
        let text = "aaaaa";
        let sa = SuffixAutomaton::new(text);
        
        assert_eq!(sa.find_all("aa"), vec![0, 1, 2, 3]);
        assert_eq!(sa.find_all("aaa"), vec![0, 1, 2]);
    }

    #[test]
    fn test_long_text() {
        let text = "a".repeat(1000) + "b";
        let sa = SuffixAutomaton::new(&text);
        
        assert!(sa.contains("aaa"));
        assert!(sa.contains("b"));
        assert!(!sa.contains("c"));
        
        let positions = sa.find_all("aa");
        assert_eq!(positions.len(), 999);
    }

    #[test]
    fn test_case_sensitivity() {
        let text = "bAnAnA";
        let sa = SuffixAutomaton::new(text);
        
        assert!(!sa.contains("ana"));
        assert!(sa.contains("AnA"));
        
        assert_eq!(sa.find_all("ana"), vec![]);
        assert_eq!(sa.find_all("AnA"), vec![1, 3]);
    }
}
