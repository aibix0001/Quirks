//! Text buffer implementation using Ropey
//!
//! The buffer is the core data structure holding the text content.
//! Uses a rope data structure for O(log n) edits in large files.
//!
//! Initial implementation by Aibotix, refined with input from Egon.

use crate::history::History;
use anyhow::Result;
use ropey::Rope;
use std::fs;
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

/// A text buffer backed by a rope data structure
#[derive(Clone)]
pub struct Buffer {
    /// The text content
    rope: Rope,
    /// Path to the file (if any)
    file_path: Option<PathBuf>,
    /// Whether the buffer has unsaved changes
    modified: bool,
    /// Undo/redo history
    history: History,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        let mut history = History::new();
        let rope = Rope::new();
        history.init(&rope, 0, 0);
        Self {
            rope,
            file_path: None,
            modified: false,
            history,
        }
    }

    /// Create a buffer from a file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let rope = Rope::from_str(&content);
        let mut history = History::new();
        history.init(&rope, 0, 0);
        Ok(Self {
            rope,
            file_path: Some(PathBuf::from(path)),
            modified: false,
            history,
        })
    }

    /// Save the buffer to its file
    pub fn save(&mut self) -> Result<()> {
        if let Some(ref path) = self.file_path {
            fs::write(path, self.rope.to_string())?;
            self.modified = false;
        }
        Ok(())
    }

    /// Save the buffer to a specific path
    pub fn save_as(&mut self, path: &str) -> Result<()> {
        fs::write(path, self.rope.to_string())?;
        self.file_path = Some(PathBuf::from(path));
        self.modified = false;
        Ok(())
    }

    /// Get the number of lines in the buffer
    pub fn line_count(&self) -> usize {
        self.rope.len_lines().max(1)
    }

    /// Get the total byte length of the buffer
    pub fn len(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Check if buffer is empty
    pub fn is_empty(&self) -> bool {
        self.rope.len_bytes() == 0
    }

    /// Get the length of a line in grapheme clusters
    pub fn line_len(&self, line_idx: usize) -> usize {
        if line_idx >= self.rope.len_lines() {
            return 0;
        }
        let line = self.rope.line(line_idx);
        line.to_string()
            .trim_end_matches('\n')
            .graphemes(true)
            .count()
    }

    /// Get a line as a string (without trailing newline)
    pub fn line(&self, line_idx: usize) -> String {
        if line_idx >= self.rope.len_lines() {
            return String::new();
        }
        self.rope
            .line(line_idx)
            .to_string()
            .trim_end_matches('\n')
            .to_string()
    }

    /// Get the byte offset of a line start
    pub fn line_to_byte(&self, line_idx: usize) -> usize {
        if line_idx >= self.rope.len_lines() {
            self.rope.len_bytes()
        } else {
            self.rope.line_to_byte(line_idx)
        }
    }

    /// Convert a grapheme column to byte offset within a line
    pub fn col_to_byte(&self, line_idx: usize, col: usize) -> usize {
        if line_idx >= self.rope.len_lines() {
            return 0;
        }
        let line = self.line(line_idx);
        line.graphemes(true)
            .take(col)
            .map(|g| g.len())
            .sum()
    }

    /// Insert a character at the given byte position
    pub fn insert_char(&mut self, byte_pos: usize, ch: char) {
        self.rope.insert_char(byte_pos, ch);
        self.modified = true;
    }

    /// Insert a string at the given byte position
    pub fn insert(&mut self, byte_pos: usize, text: &str) {
        self.rope.insert(byte_pos, text);
        self.modified = true;
    }

    /// Delete a range of bytes
    pub fn delete(&mut self, start: usize, end: usize) {
        if start < end && end <= self.rope.len_bytes() {
            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    /// Delete a single grapheme at the cursor position
    pub fn delete_grapheme(&mut self, line: usize, col: usize) {
        let line_str = self.line(line);
        let graphemes: Vec<&str> = line_str.graphemes(true).collect();
        
        if col < graphemes.len() {
            let byte_start = self.line_to_byte(line) + self.col_to_byte(line, col);
            let byte_end = byte_start + graphemes[col].len();
            self.delete(byte_start, byte_end);
        } else if col == graphemes.len() && line < self.line_count() - 1 {
            // At end of line, delete the newline to join lines
            let byte_pos = self.line_to_byte(line + 1) - 1;
            self.delete(byte_pos, byte_pos + 1);
        }
    }

    /// Delete the grapheme before the cursor (backspace)
    pub fn backspace(&mut self, line: usize, col: usize) -> (usize, usize) {
        if col > 0 {
            let line_str = self.line(line);
            let graphemes: Vec<&str> = line_str.graphemes(true).collect();
            let byte_start = self.line_to_byte(line) + self.col_to_byte(line, col - 1);
            let byte_end = byte_start + graphemes[col - 1].len();
            self.delete(byte_start, byte_end);
            (line, col - 1)
        } else if line > 0 {
            // At start of line, join with previous line
            let prev_len = self.line_len(line - 1);
            let byte_pos = self.line_to_byte(line) - 1;
            self.delete(byte_pos, byte_pos + 1);
            (line - 1, prev_len)
        } else {
            (line, col)
        }
    }

    /// Check if the buffer has been modified
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get the file name (if any)
    pub fn file_name(&self) -> Option<&str> {
        self.file_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
    }

    /// Get the full file path (if any)
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.file_path.as_ref()
    }

    /// Record current state as a checkpoint for undo
    /// Call this before making changes
    pub fn checkpoint(&mut self, cursor_line: usize, cursor_col: usize) {
        self.history.record(&self.rope, cursor_line, cursor_col);
    }

    /// Undo the last change
    /// Returns new cursor position (line, col), or None if nothing to undo
    pub fn undo(&mut self, cursor_line: usize, cursor_col: usize) -> Option<(usize, usize)> {
        if let Some((content, line, col)) = self.history.undo(&self.rope, cursor_line, cursor_col) {
            self.rope = content;
            self.modified = true;
            Some((line, col))
        } else {
            None
        }
    }

    /// Redo the last undone change
    /// Returns new cursor position (line, col), or None if nothing to redo
    pub fn redo(&mut self) -> Option<(usize, usize)> {
        if let Some((content, line, col)) = self.history.redo() {
            self.rope = content;
            self.modified = true;
            Some((line, col))
        } else {
            None
        }
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    /// Get character at position (line, col)
    pub fn char_at(&self, line: usize, col: usize) -> Option<char> {
        if line >= self.line_count() {
            return None;
        }
        let line_text = self.line(line);
        let graphemes: Vec<&str> = line_text.graphemes(true).collect();
        if col >= graphemes.len() {
            return None;
        }
        graphemes.get(col).and_then(|g| g.chars().next())
    }

    /// Delete an entire line
    pub fn delete_line(&mut self, line: usize) {
        if line >= self.line_count() {
            return;
        }
        let start = self.rope.line_to_char(line);
        let end = if line + 1 < self.line_count() {
            self.rope.line_to_char(line + 1)
        } else {
            self.rope.len_chars()
        };
        if start < end {
            self.rope.remove(start..end);
            self.modified = true;
        }
    }

    /// Insert text as a new line below the given line
    pub fn insert_line_below(&mut self, line: usize, text: &str) {
        let insert_pos = if line + 1 < self.line_count() {
            self.rope.line_to_char(line + 1)
        } else {
            // At end of file, ensure there's a newline first
            let len = self.rope.len_chars();
            if len > 0 {
                let last_char = self.rope.char(len - 1);
                if last_char != '\n' {
                    self.rope.insert_char(len, '\n');
                }
            }
            self.rope.len_chars()
        };
        self.rope.insert(insert_pos, text);
        self.modified = true;
    }

    /// Insert text as a new line above the given line
    pub fn insert_line_above(&mut self, line: usize, text: &str) {
        let insert_pos = self.rope.line_to_char(line);
        self.rope.insert(insert_pos, text);
        self.modified = true;
    }

    /// Join the next line to the current line
    pub fn join_lines(&mut self, line: usize) {
        if line + 1 >= self.line_count() {
            return;
        }
        
        // Find the newline at the end of current line
        let next_line_start = self.rope.line_to_char(line + 1);
        let newline_pos = next_line_start - 1;
        
        // Remove the newline
        self.rope.remove(newline_pos..next_line_start);
        
        // Get the (now joined) line and check if we need to add a space
        let current_line: String = self.rope.line(line).chars().collect();
        let trimmed = current_line.trim_end();
        
        // If the joined content doesn't already have a space, insert one
        if !trimmed.is_empty() && !trimmed.ends_with(' ') {
            // Find where to insert the space (after old line content)
            let insert_pos = self.rope.line_to_char(line) + trimmed.len();
            self.rope.insert_char(insert_pos, ' ');
        }
        
        self.modified = true;
    }

    /// Indent a line by adding spaces at the beginning
    pub fn indent_line(&mut self, line: usize, spaces: usize) {
        if line >= self.line_count() {
            return;
        }
        let indent: String = " ".repeat(spaces);
        let pos = self.rope.line_to_char(line);
        self.rope.insert(pos, &indent);
        self.modified = true;
    }

    /// Outdent a line by removing leading spaces
    pub fn outdent_line(&mut self, line: usize, max_spaces: usize) {
        if line >= self.line_count() {
            return;
        }
        let line_content: String = self.rope.line(line).chars().collect();
        let leading_spaces = line_content.chars().take_while(|c| *c == ' ').count();
        let to_remove = leading_spaces.min(max_spaces);
        if to_remove > 0 {
            let start = self.rope.line_to_char(line);
            self.rope.remove(start..(start + to_remove));
            self.modified = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_new() {
        let buffer = Buffer::new();
        assert_eq!(buffer.line_count(), 1);
    }

    #[test]
    fn test_buffer_insert() {
        let mut buffer = Buffer::new();
        buffer.insert(0, "Hello");
        assert!(buffer.line(0).contains("Hello"));
    }

    #[test]
    fn test_buffer_multiline() {
        let mut buffer = Buffer::new();
        buffer.insert(0, "Line 1\nLine 2\nLine 3");
        assert_eq!(buffer.line_count(), 3);
    }

    #[test]
    fn test_buffer_modified() {
        let mut buffer = Buffer::new();
        assert!(!buffer.is_modified());
        buffer.insert(0, "test");
        assert!(buffer.is_modified());
    }
}
