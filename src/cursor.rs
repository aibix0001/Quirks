//! Cursor management for Quirks
//!
//! Handles cursor position, movement, and the "sticky column" behavior
//! that makes vertical movement intuitive.

use crate::buffer::Buffer;

/// Cursor position in the document
#[derive(Debug, Clone, Copy, Default)]
pub struct Cursor {
    /// Current line (0-indexed)
    pub line: usize,
    /// Current column in grapheme clusters (0-indexed)
    pub col: usize,
    /// "Sticky" column for vertical movement - remembers the desired column
    /// when moving through lines of varying length
    sticky_col: usize,
}

impl Cursor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Move cursor left by one grapheme
    pub fn move_left(&mut self, buffer: &Buffer) {
        if self.col > 0 {
            self.col -= 1;
        } else if self.line > 0 {
            // Wrap to end of previous line
            self.line -= 1;
            self.col = buffer.line_len(self.line);
        }
        self.sticky_col = self.col;
    }

    /// Move cursor right by one grapheme
    pub fn move_right(&mut self, buffer: &Buffer) {
        let line_len = buffer.line_len(self.line);
        if self.col < line_len {
            self.col += 1;
        } else if self.line < buffer.line_count().saturating_sub(1) {
            // Wrap to start of next line
            self.line += 1;
            self.col = 0;
        }
        self.sticky_col = self.col;
    }

    /// Move cursor up one line, preserving sticky column
    pub fn move_up(&mut self, buffer: &Buffer) {
        if self.line > 0 {
            self.line -= 1;
            let line_len = buffer.line_len(self.line);
            self.col = self.sticky_col.min(line_len);
        }
    }

    /// Move cursor down one line, preserving sticky column
    pub fn move_down(&mut self, buffer: &Buffer) {
        if self.line < buffer.line_count().saturating_sub(1) {
            self.line += 1;
            let line_len = buffer.line_len(self.line);
            self.col = self.sticky_col.min(line_len);
        }
    }

    /// Move to start of current line
    pub fn move_to_line_start(&mut self) {
        self.col = 0;
        self.sticky_col = 0;
    }

    /// Move to end of current line
    pub fn move_to_line_end(&mut self, buffer: &Buffer) {
        self.col = buffer.line_len(self.line);
        self.sticky_col = self.col;
    }

    /// Move to the first line
    pub fn move_to_buffer_start(&mut self) {
        self.line = 0;
        self.col = 0;
        self.sticky_col = 0;
    }

    /// Move to the last line
    pub fn move_to_buffer_end(&mut self, buffer: &Buffer) {
        self.line = buffer.line_count().saturating_sub(1);
        self.col = buffer.line_len(self.line);
        self.sticky_col = self.col;
    }

    /// Ensure cursor is within buffer bounds
    pub fn clamp(&mut self, buffer: &Buffer) {
        let line_count = buffer.line_count();
        if line_count == 0 {
            self.line = 0;
            self.col = 0;
        } else {
            self.line = self.line.min(line_count - 1);
            self.col = self.col.min(buffer.line_len(self.line));
        }
    }

    /// Get the byte offset in the buffer for the current cursor position
    pub fn byte_offset(&self, buffer: &Buffer) -> usize {
        buffer.line_to_byte(self.line) + buffer.col_to_byte(self.line, self.col)
    }
}
