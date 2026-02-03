//! Text buffer implementation using Rope data structure.
//!
//! Provides O(log n) insert/delete operations for large files.

use ropey::Rope;
use std::fs;
use std::io::{self, BufReader, BufWriter};
use std::path::{Path, PathBuf};

/// A text buffer backed by a Rope data structure.
#[derive(Debug)]
pub struct Buffer {
    /// The text content
    content: Rope,
    /// Associated file path (if any)
    filepath: Option<PathBuf>,
    /// Whether the buffer has unsaved changes
    modified: bool,
}

impl Buffer {
    /// Create a new empty buffer.
    pub fn new() -> Self {
        Self {
            content: Rope::new(),
            filepath: None,
            modified: false,
        }
    }

    /// Create a buffer from a string.
    pub fn from_str(text: &str) -> Self {
        Self {
            content: Rope::from_str(text),
            filepath: None,
            modified: false,
        }
    }

    /// Load a buffer from a file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path = path.as_ref();
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let content = Rope::from_reader(reader)?;
        
        Ok(Self {
            content,
            filepath: Some(path.to_path_buf()),
            modified: false,
        })
    }

    /// Save the buffer to its associated file.
    pub fn save(&mut self) -> io::Result<()> {
        let path = self.filepath.as_ref()
            .ok_or_else(|| io::Error::new(
                io::ErrorKind::NotFound,
                "No file path associated with buffer"
            ))?;
        
        let file = fs::File::create(path)?;
        let mut writer = BufWriter::new(file);
        self.content.write_to(&mut writer)?;
        self.modified = false;
        Ok(())
    }

    /// Save the buffer to a specific path.
    pub fn save_as<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        self.filepath = Some(path.as_ref().to_path_buf());
        self.save()
    }

    // === Getters ===

    /// Get the total number of lines.
    pub fn line_count(&self) -> usize {
        self.content.len_lines()
    }

    /// Get the total number of characters.
    pub fn char_count(&self) -> usize {
        self.content.len_chars()
    }

    /// Get a specific line (0-indexed).
    pub fn line(&self, idx: usize) -> Option<ropey::RopeSlice> {
        if idx < self.line_count() {
            Some(self.content.line(idx))
        } else {
            None
        }
    }

    /// Get the file path, if any.
    pub fn filepath(&self) -> Option<&Path> {
        self.filepath.as_deref()
    }

    /// Check if the buffer has unsaved modifications.
    pub fn is_modified(&self) -> bool {
        self.modified
    }

    /// Get a reference to the underlying Rope.
    pub fn content(&self) -> &Rope {
        &self.content
    }

    // === Editing Operations ===

    /// Insert text at a character index.
    pub fn insert(&mut self, char_idx: usize, text: &str) {
        if char_idx <= self.char_count() {
            self.content.insert(char_idx, text);
            self.modified = true;
        }
    }

    /// Insert a single character at a character index.
    pub fn insert_char(&mut self, char_idx: usize, ch: char) {
        if char_idx <= self.char_count() {
            self.content.insert_char(char_idx, ch);
            self.modified = true;
        }
    }

    /// Delete a range of characters.
    pub fn delete(&mut self, start: usize, end: usize) {
        let end = end.min(self.char_count());
        let start = start.min(end);
        if start < end {
            self.content.remove(start..end);
            self.modified = true;
        }
    }

    /// Delete a single character at index.
    pub fn delete_char(&mut self, char_idx: usize) {
        if char_idx < self.char_count() {
            self.content.remove(char_idx..char_idx + 1);
            self.modified = true;
        }
    }

    // === Coordinate Conversion ===

    /// Convert (line, column) to character index.
    pub fn line_col_to_char(&self, line: usize, col: usize) -> Option<usize> {
        if line >= self.line_count() {
            return None;
        }
        let line_start = self.content.line_to_char(line);
        let line_len = self.content.line(line).len_chars();
        let col = col.min(line_len.saturating_sub(1));
        Some(line_start + col)
    }

    /// Convert character index to (line, column).
    pub fn char_to_line_col(&self, char_idx: usize) -> (usize, usize) {
        let char_idx = char_idx.min(self.char_count().saturating_sub(1));
        let line = self.content.char_to_line(char_idx);
        let line_start = self.content.line_to_char(line);
        let col = char_idx - line_start;
        (line, col)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_buffer() {
        let buf = Buffer::new();
        assert_eq!(buf.char_count(), 0);
        assert_eq!(buf.line_count(), 1); // Empty buffer has 1 line
        assert!(!buf.is_modified());
    }

    #[test]
    fn test_from_str() {
        let buf = Buffer::from_str("Hello\nWorld");
        assert_eq!(buf.line_count(), 2);
        assert_eq!(buf.char_count(), 11);
    }

    #[test]
    fn test_insert() {
        let mut buf = Buffer::new();
        buf.insert(0, "Hello");
        assert_eq!(buf.char_count(), 5);
        assert!(buf.is_modified());
    }

    #[test]
    fn test_delete() {
        let mut buf = Buffer::from_str("Hello World");
        buf.delete(5, 11); // Delete " World"
        assert_eq!(buf.content().to_string(), "Hello");
    }

    #[test]
    fn test_line_col_conversion() {
        let buf = Buffer::from_str("Hello\nWorld\n!");
        assert_eq!(buf.char_to_line_col(0), (0, 0));
        assert_eq!(buf.char_to_line_col(6), (1, 0)); // 'W'
        assert_eq!(buf.line_col_to_char(1, 0), Some(6));
    }
}
