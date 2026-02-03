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

    /// Move to the start of the next word (w)
    pub fn move_word_forward(&mut self, buffer: &Buffer) {
        let total_lines = buffer.line_count();
        if total_lines == 0 {
            return;
        }

        let mut line = self.line;
        let mut col = self.col;

        loop {
            let line_content = buffer.line(line);
            let chars: Vec<char> = line_content.chars().collect();

            // Skip current word (non-whitespace of same type)
            if col < chars.len() {
                let start_type = char_type(chars[col]);
                while col < chars.len() && char_type(chars[col]) == start_type {
                    col += 1;
                }
            }

            // Skip whitespace
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }

            // If we found a word start, we're done
            if col < chars.len() {
                self.line = line;
                self.col = col;
                self.sticky_col = col;
                return;
            }

            // Move to next line
            if line + 1 < total_lines {
                line += 1;
                col = 0;
                // Skip leading whitespace on new line
                let next_line = buffer.line(line);
                let next_chars: Vec<char> = next_line.chars().collect();
                while col < next_chars.len() && next_chars[col].is_whitespace() {
                    col += 1;
                }
                if col < next_chars.len() {
                    self.line = line;
                    self.col = col;
                    self.sticky_col = col;
                    return;
                }
            } else {
                // End of buffer
                self.line = line;
                self.col = chars.len().saturating_sub(1);
                self.sticky_col = self.col;
                return;
            }
        }
    }

    /// Move to the start of the previous word (b)
    pub fn move_word_backward(&mut self, buffer: &Buffer) {
        let total_lines = buffer.line_count();
        if total_lines == 0 {
            return;
        }

        let mut line = self.line;
        let mut col = self.col;

        loop {
            // Move back one if at start of word
            if col > 0 {
                col -= 1;
            } else if line > 0 {
                line -= 1;
                let prev_line = buffer.line(line);
                col = prev_line.chars().count().saturating_sub(1);
                continue;
            } else {
                self.line = 0;
                self.col = 0;
                self.sticky_col = 0;
                return;
            }

            let chars: Vec<char> = buffer.line(line).chars().collect();
            
            // Skip whitespace going backward
            while col > 0 && chars.get(col).map_or(false, |c| c.is_whitespace()) {
                col -= 1;
            }

            // If we hit whitespace at col 0, try previous line
            if chars.get(col).map_or(false, |c| c.is_whitespace()) {
                if line > 0 {
                    line -= 1;
                    let prev_line = buffer.line(line);
                    col = prev_line.chars().count().saturating_sub(1);
                    continue;
                } else {
                    self.line = 0;
                    self.col = 0;
                    self.sticky_col = 0;
                    return;
                }
            }

            // Find start of current word
            let word_type = char_type(chars[col]);
            while col > 0 && char_type(chars[col - 1]) == word_type {
                col -= 1;
            }

            self.line = line;
            self.col = col;
            self.sticky_col = col;
            return;
        }
    }

    /// Move to the end of the current/next word (e)
    pub fn move_word_end(&mut self, buffer: &Buffer) {
        let total_lines = buffer.line_count();
        if total_lines == 0 {
            return;
        }

        let mut line = self.line;
        let mut col = self.col;

        // Move forward by one first (so we can find next word end if at end)
        let line_content = buffer.line(line);
        let chars: Vec<char> = line_content.chars().collect();
        if col + 1 < chars.len() {
            col += 1;
        } else if line + 1 < total_lines {
            line += 1;
            col = 0;
        }

        loop {
            let line_content = buffer.line(line);
            let chars: Vec<char> = line_content.chars().collect();

            // Skip whitespace
            while col < chars.len() && chars[col].is_whitespace() {
                col += 1;
            }

            // If at end of line, try next line
            if col >= chars.len() {
                if line + 1 < total_lines {
                    line += 1;
                    col = 0;
                    continue;
                } else {
                    self.line = line;
                    self.col = chars.len().saturating_sub(1);
                    self.sticky_col = self.col;
                    return;
                }
            }

            // Find end of word
            let word_type = char_type(chars[col]);
            while col + 1 < chars.len() && char_type(chars[col + 1]) == word_type {
                col += 1;
            }

            self.line = line;
            self.col = col;
            self.sticky_col = col;
            return;
        }
    }
}

/// Character classification for word motion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CharType {
    Word,       // Alphanumeric + underscore
    Punctuation, // Other non-whitespace
    Whitespace,
}

fn char_type(c: char) -> CharType {
    if c.is_whitespace() {
        CharType::Whitespace
    } else if c.is_alphanumeric() || c == '_' {
        CharType::Word
    } else {
        CharType::Punctuation
    }
}
