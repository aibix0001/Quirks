//! Search functionality for Quirks.
//!
//! Implements Vim-style search with `/` (forward) and `?` (backward).

use crate::buffer::Buffer;
use crate::cursor::Cursor;

/// Search direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDirection {
    Forward,
    Backward,
}

/// Search state for the editor
#[derive(Debug)]
pub struct Search {
    /// Current search pattern
    pattern: String,
    /// Last search direction
    direction: SearchDirection,
    /// All matches in the buffer (line, start_col, end_col)
    matches: Vec<Match>,
    /// Current match index
    current_match: Option<usize>,
    /// Whether search is case-sensitive
    case_sensitive: bool,
}

/// A single search match
#[derive(Debug, Clone, Copy)]
pub struct Match {
    pub line: usize,
    pub start_col: usize,
    pub end_col: usize,
}

impl Search {
    pub fn new() -> Self {
        Self {
            pattern: String::new(),
            direction: SearchDirection::Forward,
            matches: Vec::new(),
            current_match: None,
            case_sensitive: false,
        }
    }

    /// Get the current search pattern
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Check if there's an active search
    pub fn is_active(&self) -> bool {
        !self.pattern.is_empty()
    }

    /// Get all matches
    pub fn matches(&self) -> &[Match] {
        &self.matches
    }

    /// Get current match index
    pub fn current_match_index(&self) -> Option<usize> {
        self.current_match
    }

    /// Get total match count
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Set case sensitivity
    pub fn set_case_sensitive(&mut self, sensitive: bool) {
        self.case_sensitive = sensitive;
    }

    /// Execute a search from the current cursor position
    pub fn search(&mut self, pattern: &str, direction: SearchDirection, buffer: &Buffer) {
        self.pattern = pattern.to_string();
        self.direction = direction;
        self.find_all_matches(buffer);
    }

    /// Find all occurrences of the pattern in the buffer
    fn find_all_matches(&mut self, buffer: &Buffer) {
        self.matches.clear();
        self.current_match = None;

        if self.pattern.is_empty() {
            return;
        }

        let search_pattern = if self.case_sensitive {
            self.pattern.clone()
        } else {
            self.pattern.to_lowercase()
        };

        for line_idx in 0..buffer.line_count() {
            if let Some(line) = buffer.line(line_idx) {
                let line_str = line.to_string();
                let search_line = if self.case_sensitive {
                    line_str.clone()
                } else {
                    line_str.to_lowercase()
                };

                let mut start = 0;
                while let Some(pos) = search_line[start..].find(&search_pattern) {
                    let match_start = start + pos;
                    let match_end = match_start + search_pattern.len();
                    
                    self.matches.push(Match {
                        line: line_idx,
                        start_col: match_start,
                        end_col: match_end,
                    });
                    
                    start = match_start + 1;
                    if start >= search_line.len() {
                        break;
                    }
                }
            }
        }
    }

    /// Find and jump to the next match from cursor position
    pub fn next_match(&mut self, cursor: &Cursor, buffer: &Buffer) -> Option<Match> {
        if self.matches.is_empty() {
            return None;
        }

        let cursor_line = cursor.line;
        let cursor_col = cursor.col;

        // Find the next match after cursor
        let next_idx = match self.direction {
            SearchDirection::Forward => {
                self.matches.iter().position(|m| {
                    m.line > cursor_line || (m.line == cursor_line && m.start_col > cursor_col)
                }).or_else(|| Some(0)) // Wrap around
            }
            SearchDirection::Backward => {
                self.matches.iter().rposition(|m| {
                    m.line < cursor_line || (m.line == cursor_line && m.start_col < cursor_col)
                }).or_else(|| Some(self.matches.len() - 1)) // Wrap around
            }
        };

        if let Some(idx) = next_idx {
            self.current_match = Some(idx);
            Some(self.matches[idx])
        } else {
            None
        }
    }

    /// Jump to next match (n command)
    pub fn jump_next(&mut self, cursor: &Cursor) -> Option<Match> {
        if self.matches.is_empty() {
            return None;
        }

        let current = self.current_match.unwrap_or(0);
        let next = match self.direction {
            SearchDirection::Forward => {
                if current + 1 >= self.matches.len() {
                    0 // Wrap
                } else {
                    current + 1
                }
            }
            SearchDirection::Backward => {
                if current == 0 {
                    self.matches.len() - 1 // Wrap
                } else {
                    current - 1
                }
            }
        };

        self.current_match = Some(next);
        Some(self.matches[next])
    }

    /// Jump to previous match (N command)
    pub fn jump_prev(&mut self, cursor: &Cursor) -> Option<Match> {
        if self.matches.is_empty() {
            return None;
        }

        let current = self.current_match.unwrap_or(0);
        let prev = match self.direction {
            SearchDirection::Forward => {
                // Opposite of normal direction
                if current == 0 {
                    self.matches.len() - 1
                } else {
                    current - 1
                }
            }
            SearchDirection::Backward => {
                if current + 1 >= self.matches.len() {
                    0
                } else {
                    current + 1
                }
            }
        };

        self.current_match = Some(prev);
        Some(self.matches[prev])
    }

    /// Clear the search
    pub fn clear(&mut self) {
        self.pattern.clear();
        self.matches.clear();
        self.current_match = None;
    }

    /// Check if a position is within a match (for highlighting)
    pub fn is_match_at(&self, line: usize, col: usize) -> bool {
        self.matches.iter().any(|m| {
            m.line == line && col >= m.start_col && col < m.end_col
        })
    }

    /// Check if a position is the current match (for special highlighting)
    pub fn is_current_match_at(&self, line: usize, col: usize) -> bool {
        if let Some(idx) = self.current_match {
            let m = &self.matches[idx];
            m.line == line && col >= m.start_col && col < m.end_col
        } else {
            false
        }
    }
}

impl Default for Search {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_forward() {
        let buffer = Buffer::from_str("hello world\nhello again\nworld hello");
        let mut search = Search::new();
        
        search.search("hello", SearchDirection::Forward, &buffer);
        
        assert_eq!(search.match_count(), 3);
        assert_eq!(search.matches()[0].line, 0);
        assert_eq!(search.matches()[0].start_col, 0);
        assert_eq!(search.matches()[1].line, 1);
        assert_eq!(search.matches()[2].line, 2);
    }

    #[test]
    fn test_search_case_insensitive() {
        let buffer = Buffer::from_str("Hello HELLO hello");
        let mut search = Search::new();
        search.set_case_sensitive(false);
        
        search.search("hello", SearchDirection::Forward, &buffer);
        
        assert_eq!(search.match_count(), 3);
    }

    #[test]
    fn test_search_case_sensitive() {
        let buffer = Buffer::from_str("Hello HELLO hello");
        let mut search = Search::new();
        search.set_case_sensitive(true);
        
        search.search("hello", SearchDirection::Forward, &buffer);
        
        assert_eq!(search.match_count(), 1);
    }

    #[test]
    fn test_next_match_wraps() {
        let buffer = Buffer::from_str("test\ntest\ntest");
        let mut search = Search::new();
        search.search("test", SearchDirection::Forward, &buffer);
        
        // Start at last match
        search.current_match = Some(2);
        
        let next = search.jump_next(&Cursor::new());
        assert!(next.is_some());
        assert_eq!(search.current_match, Some(0)); // Wrapped to first
    }

    #[test]
    fn test_is_match_at() {
        let buffer = Buffer::from_str("hello world");
        let mut search = Search::new();
        search.search("world", SearchDirection::Forward, &buffer);
        
        assert!(!search.is_match_at(0, 0)); // 'h'
        assert!(search.is_match_at(0, 6));  // 'w' in "world"
        assert!(search.is_match_at(0, 10)); // 'd' in "world"
        assert!(!search.is_match_at(0, 11)); // past end
    }

    #[test]
    fn test_clear_search() {
        let buffer = Buffer::from_str("test");
        let mut search = Search::new();
        search.search("test", SearchDirection::Forward, &buffer);
        
        assert!(search.is_active());
        search.clear();
        assert!(!search.is_active());
        assert_eq!(search.match_count(), 0);
    }
}
