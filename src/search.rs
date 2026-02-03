//! Search functionality for Quirks
//!
//! Provides vi-style search with regex support.

use regex::Regex;

/// Search direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// A search match location
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SearchMatch {
    /// Line number (0-indexed)
    pub line: usize,
    /// Start column in the line (0-indexed, in chars)
    pub start_col: usize,
    /// End column (exclusive)
    pub end_col: usize,
}

/// Search state
#[derive(Debug)]
pub struct Search {
    /// Current search pattern (as entered by user)
    pattern: String,
    /// Compiled regex (if valid)
    regex: Option<Regex>,
    /// Search direction
    direction: SearchDirection,
    /// All matches in the buffer
    matches: Vec<SearchMatch>,
    /// Current match index
    current_match: Option<usize>,
    /// Whether search highlighting is active
    pub highlight_active: bool,
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

impl Search {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            regex: None,
            direction: SearchDirection::Forward,
            matches: Vec::new(),
            current_match: None,
            highlight_active: false,
        }
    }

    /// Start a new search
    pub fn start(&mut self, direction: SearchDirection) {
        self.direction = direction;
        self.pattern.clear();
        self.regex = None;
        self.matches.clear();
        self.current_match = None;
    }

    /// Get the current pattern
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Set the pattern and compile regex
    pub fn set_pattern(&mut self, pattern: &str) {
        self.pattern = pattern.to_string();
        // Try to compile as regex, fall back to literal if invalid
        self.regex = Regex::new(pattern).ok().or_else(|| {
            Regex::new(&regex::escape(pattern)).ok()
        });
    }

    /// Add a character to the pattern
    pub fn push_char(&mut self, c: char) {
        self.pattern.push(c);
        self.regex = Regex::new(&self.pattern).ok().or_else(|| {
            Regex::new(&regex::escape(&self.pattern)).ok()
        });
    }

    /// Remove last character from pattern
    pub fn pop_char(&mut self) -> bool {
        if self.pattern.pop().is_some() {
            self.regex = if self.pattern.is_empty() {
                None
            } else {
                Regex::new(&self.pattern).ok().or_else(|| {
                    Regex::new(&regex::escape(&self.pattern)).ok()
                })
            };
            true
        } else {
            false
        }
    }

    /// Execute search on buffer content
    pub fn execute(&mut self, lines: &[String], cursor_line: usize, cursor_col: usize) {
        self.matches.clear();
        self.current_match = None;
        
        let regex = match &self.regex {
            Some(r) => r,
            None => return,
        };

        // Find all matches
        for (line_idx, line) in lines.iter().enumerate() {
            for mat in regex.find_iter(line) {
                self.matches.push(SearchMatch {
                    line: line_idx,
                    start_col: line[..mat.start()].chars().count(),
                    end_col: line[..mat.end()].chars().count(),
                });
            }
        }

        if self.matches.is_empty() {
            return;
        }

        self.highlight_active = true;

        // Find the nearest match based on direction
        self.current_match = Some(self.find_nearest_match(cursor_line, cursor_col));
    }

    /// Find the nearest match from cursor position
    fn find_nearest_match(&self, cursor_line: usize, cursor_col: usize) -> usize {
        if self.matches.is_empty() {
            return 0;
        }

        match self.direction {
            SearchDirection::Forward => {
                // Find first match at or after cursor
                for (i, m) in self.matches.iter().enumerate() {
                    if m.line > cursor_line || (m.line == cursor_line && m.start_col >= cursor_col) {
                        return i;
                    }
                }
                // Wrap around to first match
                0
            }
            SearchDirection::Backward => {
                // Find first match before cursor
                for (i, m) in self.matches.iter().enumerate().rev() {
                    if m.line < cursor_line || (m.line == cursor_line && m.start_col < cursor_col) {
                        return i;
                    }
                }
                // Wrap around to last match
                self.matches.len().saturating_sub(1)
            }
        }
    }

    /// Go to next match
    pub fn next_match(&mut self) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let next = match self.current_match {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.current_match = Some(next);
        self.matches.get(next).copied()
    }

    /// Go to previous match
    pub fn prev_match(&mut self) -> Option<SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }

        let prev = match self.current_match {
            Some(i) => {
                if i == 0 {
                    self.matches.len() - 1
                } else {
                    i - 1
                }
            }
            None => self.matches.len() - 1,
        };
        self.current_match = Some(prev);
        self.matches.get(prev).copied()
    }

    /// Get current match
    pub fn current(&self) -> Option<SearchMatch> {
        self.current_match.and_then(|i| self.matches.get(i).copied())
    }

    /// Get all matches (for highlighting)
    pub fn matches(&self) -> &[SearchMatch] {
        &self.matches
    }

    /// Get match count info string
    pub fn match_info(&self) -> String {
        if self.matches.is_empty() {
            if self.pattern.is_empty() {
                String::new()
            } else {
                "No matches".to_string()
            }
        } else {
            let current = self.current_match.map(|i| i + 1).unwrap_or(0);
            format!("{}/{}", current, self.matches.len())
        }
    }

    /// Get search direction
    pub fn direction(&self) -> SearchDirection {
        self.direction
    }

    /// Clear search highlighting
    pub fn clear_highlight(&mut self) {
        self.highlight_active = false;
    }

    /// Check if pattern is empty
    pub fn is_empty(&self) -> bool {
        self.pattern.is_empty()
    }
}
