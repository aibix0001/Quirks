//! Selection and Visual mode for Quirks
//!
//! Supports character-wise, line-wise, and block selection.

/// Visual mode type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualMode {
    /// Character-wise selection (v)
    Char,
    /// Line-wise selection (V)
    Line,
    /// Block/column selection (Ctrl+V)
    Block,
}

/// A text selection
#[derive(Debug, Clone, Copy)]
pub struct Selection {
    /// Visual mode type
    pub mode: VisualMode,
    /// Anchor position (where selection started)
    pub anchor_line: usize,
    pub anchor_col: usize,
    /// Current position (where cursor is now)
    pub cursor_line: usize,
    pub cursor_col: usize,
}

impl Selection {
    /// Create a new selection starting at the given position
    pub fn new(mode: VisualMode, line: usize, col: usize) -> Self {
        Self {
            mode,
            anchor_line: line,
            anchor_col: col,
            cursor_line: line,
            cursor_col: col,
        }
    }

    /// Update the cursor end of the selection
    pub fn update_cursor(&mut self, line: usize, col: usize) {
        self.cursor_line = line;
        self.cursor_col = col;
    }

    /// Get the normalized selection range (start <= end)
    pub fn normalized(&self) -> (usize, usize, usize, usize) {
        let (start_line, end_line) = if self.anchor_line <= self.cursor_line {
            (self.anchor_line, self.cursor_line)
        } else {
            (self.cursor_line, self.anchor_line)
        };

        let (start_col, end_col) = if self.anchor_line == self.cursor_line {
            if self.anchor_col <= self.cursor_col {
                (self.anchor_col, self.cursor_col)
            } else {
                (self.cursor_col, self.anchor_col)
            }
        } else if self.anchor_line < self.cursor_line {
            (self.anchor_col, self.cursor_col)
        } else {
            (self.cursor_col, self.anchor_col)
        };

        (start_line, start_col, end_line, end_col)
    }

    /// Check if a position is within the selection
    pub fn contains(&self, line: usize, col: usize) -> bool {
        match self.mode {
            VisualMode::Char => self.contains_char(line, col),
            VisualMode::Line => self.contains_line(line),
            VisualMode::Block => self.contains_block(line, col),
        }
    }

    fn contains_char(&self, line: usize, col: usize) -> bool {
        let (start_line, start_col, end_line, end_col) = self.normalized();
        
        if line < start_line || line > end_line {
            return false;
        }
        
        if start_line == end_line {
            // Single line selection
            col >= start_col && col <= end_col
        } else if line == start_line {
            col >= start_col
        } else if line == end_line {
            col <= end_col
        } else {
            true // Middle lines are fully selected
        }
    }

    fn contains_line(&self, line: usize) -> bool {
        let (start_line, _, end_line, _) = self.normalized();
        line >= start_line && line <= end_line
    }

    fn contains_block(&self, line: usize, col: usize) -> bool {
        let (start_line, _, end_line, _) = self.normalized();
        let (left_col, right_col) = if self.anchor_col <= self.cursor_col {
            (self.anchor_col, self.cursor_col)
        } else {
            (self.cursor_col, self.anchor_col)
        };
        
        line >= start_line && line <= end_line && col >= left_col && col <= right_col
    }

    /// Get selected line range (inclusive)
    pub fn line_range(&self) -> (usize, usize) {
        let (start_line, _, end_line, _) = self.normalized();
        (start_line, end_line)
    }

    /// Get column range for block mode
    pub fn col_range(&self) -> (usize, usize) {
        if self.anchor_col <= self.cursor_col {
            (self.anchor_col, self.cursor_col)
        } else {
            (self.cursor_col, self.anchor_col)
        }
    }
}

/// Clipboard/register for yanked text
#[derive(Debug, Clone, Default)]
pub struct Register {
    /// The yanked content
    pub content: String,
    /// Whether this was a line-wise yank
    pub linewise: bool,
}

impl Register {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, content: String, linewise: bool) {
        self.content = content;
        self.linewise = linewise;
    }

    pub fn get(&self) -> (&str, bool) {
        (&self.content, self.linewise)
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_new() {
        let sel = Selection::new(VisualMode::Char, 5, 10);
        assert_eq!(sel.anchor_line, 5);
        assert_eq!(sel.anchor_col, 10);
        assert_eq!(sel.cursor_line, 5);
        assert_eq!(sel.cursor_col, 10);
    }

    #[test]
    fn test_selection_update_cursor() {
        let mut sel = Selection::new(VisualMode::Char, 0, 0);
        sel.update_cursor(5, 10);
        assert_eq!(sel.cursor_line, 5);
        assert_eq!(sel.cursor_col, 10);
    }

    #[test]
    fn test_selection_normalized() {
        let mut sel = Selection::new(VisualMode::Char, 5, 10);
        sel.update_cursor(2, 3);
        let (start_line, start_col, end_line, end_col) = sel.normalized();
        assert_eq!(start_line, 2);
        assert_eq!(start_col, 3);
        assert_eq!(end_line, 5);
        assert_eq!(end_col, 10);
    }

    #[test]
    fn test_visual_mode_variants() {
        let char_mode = VisualMode::Char;
        let line_mode = VisualMode::Line;
        let block_mode = VisualMode::Block;
        
        // Just ensure they can be created and compared
        assert_ne!(char_mode, line_mode);
        assert_ne!(line_mode, block_mode);
    }

    #[test]
    fn test_register_new() {
        let reg = Register::new();
        assert!(reg.is_empty());
    }

    #[test]
    fn test_register_set_get() {
        let mut reg = Register::new();
        reg.set("hello".to_string(), false);
        let (content, linewise) = reg.get();
        assert_eq!(content, "hello");
        assert!(!linewise);
    }
}
