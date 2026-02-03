//! Undo/Redo history for Quirks
//!
//! Stores buffer snapshots for undo/redo operations.
//! Uses a simple snapshot approach - each edit creates a checkpoint.

use ropey::Rope;

/// Maximum number of undo states to keep
const MAX_HISTORY_SIZE: usize = 1000;

/// A snapshot of the buffer state
#[derive(Clone)]
struct Snapshot {
    /// The text content
    content: Rope,
    /// Cursor line at this snapshot
    cursor_line: usize,
    /// Cursor column at this snapshot
    cursor_col: usize,
}

/// Undo/Redo history manager
#[derive(Clone)]
pub struct History {
    /// Past states (for undo)
    undo_stack: Vec<Snapshot>,
    /// Future states (for redo)
    redo_stack: Vec<Snapshot>,
    /// Current content (for change detection)
    current_content: Option<Rope>,
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            current_content: None,
        }
    }

    /// Initialize with the current buffer content
    pub fn init(&mut self, content: &Rope, cursor_line: usize, cursor_col: usize) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_content = Some(content.clone());
        // Save initial state so we can undo back to it
        self.undo_stack.push(Snapshot {
            content: content.clone(),
            cursor_line,
            cursor_col,
        });
    }

    /// Record a change to the buffer
    /// Call this BEFORE making the change, with the current state
    pub fn record(&mut self, content: &Rope, cursor_line: usize, cursor_col: usize) {
        // Clear redo stack on new edit
        self.redo_stack.clear();
        
        // Only record if content actually changed
        if let Some(ref current) = self.current_content {
            if content.len_bytes() == current.len_bytes() {
                // Quick length check - if same length, do full comparison
                let content_str = content.to_string();
                let current_str = current.to_string();
                if content_str == current_str {
                    return; // No change
                }
            }
        }
        
        // Save current state to undo stack
        self.undo_stack.push(Snapshot {
            content: content.clone(),
            cursor_line,
            cursor_col,
        });
        
        // Limit history size
        if self.undo_stack.len() > MAX_HISTORY_SIZE {
            self.undo_stack.remove(0);
        }
        
        self.current_content = Some(content.clone());
    }

    /// Update the current content without recording (for tracking)
    pub fn update_current(&mut self, content: &Rope) {
        self.current_content = Some(content.clone());
    }

    /// Undo the last change
    /// Returns the state to restore (content, cursor_line, cursor_col), or None if nothing to undo
    pub fn undo(&mut self, current_content: &Rope, cursor_line: usize, cursor_col: usize) -> Option<(Rope, usize, usize)> {
        if self.undo_stack.len() <= 1 {
            // Need at least 2 states: initial + one change
            return None;
        }
        
        // Save current state to redo stack
        self.redo_stack.push(Snapshot {
            content: current_content.clone(),
            cursor_line,
            cursor_col,
        });
        
        // Pop and discard current state from undo stack
        self.undo_stack.pop();
        
        // Get previous state
        if let Some(snapshot) = self.undo_stack.last() {
            self.current_content = Some(snapshot.content.clone());
            Some((snapshot.content.clone(), snapshot.cursor_line, snapshot.cursor_col))
        } else {
            None
        }
    }

    /// Redo the last undone change
    /// Returns the state to restore, or None if nothing to redo
    pub fn redo(&mut self) -> Option<(Rope, usize, usize)> {
        if let Some(snapshot) = self.redo_stack.pop() {
            // Add to undo stack
            self.undo_stack.push(snapshot.clone());
            self.current_content = Some(snapshot.content.clone());
            Some((snapshot.content, snapshot.cursor_line, snapshot.cursor_col))
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.undo_stack.len() > 1
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get undo stack size (for status display)
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len().saturating_sub(1)
    }

    /// Get redo stack size (for status display)
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ropey::Rope;

    #[test]
    fn test_history_new() {
        let history = History::new();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_history_clear() {
        let mut history = History::new();
        let rope = Rope::from_str("test");
        history.init(&rope, 0, 0);
        let rope2 = Rope::from_str("test2");
        history.record(&rope2, 0, 1);
        // After clear, no undo should be possible
        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }

    #[test]
    fn test_undo_redo_count() {
        let history = History::new();
        assert_eq!(history.undo_count(), 0);
        assert_eq!(history.redo_count(), 0);
    }
}
