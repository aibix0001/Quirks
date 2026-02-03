//! Text buffer implementation using Ropey
//!
//! The buffer is the core data structure holding the text content.
//! Uses a rope data structure for O(log n) edits in large files.
//!
//! Initial implementation by Aibotix, refined with input from Egon.

use anyhow::Result;
use ropey::Rope;
use std::fs;
use std::path::PathBuf;
use unicode_segmentation::UnicodeSegmentation;

/// A text buffer backed by a rope data structure
#[derive(Debug)]
pub struct Buffer {
    /// The text content
    rope: Rope,
    /// Path to the file (if any)
    file_path: Option<PathBuf>,
    /// Whether the buffer has unsaved changes
    modified: bool,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    /// Create a new empty buffer
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            file_path: None,
            modified: false,
        }
    }

    /// Create a buffer from a file
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Ok(Self {
            rope: Rope::from_str(&content),
            file_path: Some(PathBuf::from(path)),
            modified: false,
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
}
