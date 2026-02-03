//! Vim-style register system for yank/paste operations.
//!
//! Registers store yanked/deleted text for later pasting.

use std::collections::HashMap;

/// The type of content stored in a register
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterContent {
    /// Character-wise content (from y, d, x, etc.)
    Chars(String),
    /// Line-wise content (from yy, dd, etc.)
    Lines(String),
    /// Block content (from visual block mode) - future
    Block(Vec<String>),
}

impl RegisterContent {
    /// Get the text content
    pub fn text(&self) -> &str {
        match self {
            RegisterContent::Chars(s) => s,
            RegisterContent::Lines(s) => s,
            RegisterContent::Block(lines) => {
                // For block, return first line (simplified)
                lines.first().map(|s| s.as_str()).unwrap_or("")
            }
        }
    }

    /// Check if content is line-wise
    pub fn is_linewise(&self) -> bool {
        matches!(self, RegisterContent::Lines(_))
    }

    /// Check if content is empty
    pub fn is_empty(&self) -> bool {
        match self {
            RegisterContent::Chars(s) => s.is_empty(),
            RegisterContent::Lines(s) => s.is_empty(),
            RegisterContent::Block(lines) => lines.is_empty(),
        }
    }
}

/// The register bank holds all named and special registers
#[derive(Debug)]
pub struct Registers {
    /// Named registers (a-z, A-Z appends to a-z)
    named: HashMap<char, RegisterContent>,
    /// The unnamed register (") - default for yank/delete
    unnamed: Option<RegisterContent>,
    /// The small delete register (-) - for deletes less than one line
    small_delete: Option<RegisterContent>,
    /// Numbered registers (0-9)
    /// 0 = last yank, 1-9 = last deletes (1 is most recent)
    numbered: [Option<RegisterContent>; 10],
    // Note: The black hole register (_) discards content - we don't store anything
    // Read-only registers (%, #, etc.) would be computed on access
}

impl Registers {
    pub fn new() -> Self {
        Self {
            named: HashMap::new(),
            unnamed: None,
            small_delete: None,
            numbered: Default::default(),
        }
    }

    /// Set the unnamed register (used by most yank/delete operations)
    pub fn set_unnamed(&mut self, content: RegisterContent) {
        self.unnamed = Some(content);
    }

    /// Get the unnamed register
    pub fn get_unnamed(&self) -> Option<&RegisterContent> {
        self.unnamed.as_ref()
    }

    /// Set a named register (a-z)
    /// Uppercase (A-Z) appends to the lowercase version
    pub fn set_named(&mut self, name: char, content: RegisterContent) {
        if name.is_ascii_uppercase() {
            // Append to existing
            let lower = name.to_ascii_lowercase();
            if let Some(existing) = self.named.get_mut(&lower) {
                match (existing, &content) {
                    (RegisterContent::Chars(ref mut s), RegisterContent::Chars(new)) => {
                        s.push_str(new);
                    }
                    (RegisterContent::Lines(ref mut s), RegisterContent::Lines(new)) => {
                        s.push_str(new);
                    }
                    _ => {
                        // Type mismatch - replace
                        self.named.insert(lower, content);
                    }
                }
            } else {
                self.named.insert(lower, content);
            }
        } else if name.is_ascii_lowercase() {
            self.named.insert(name, content);
        }
    }

    /// Get a named register
    pub fn get_named(&self, name: char) -> Option<&RegisterContent> {
        let lower = name.to_ascii_lowercase();
        self.named.get(&lower)
    }

    /// Store a yank operation (goes to " and 0)
    pub fn yank(&mut self, content: RegisterContent) {
        self.numbered[0] = Some(content.clone());
        self.unnamed = Some(content);
    }

    /// Store a delete operation (goes to " and shifts 1-9)
    pub fn delete(&mut self, content: RegisterContent) {
        // Check if small delete (less than one line, char-wise)
        let is_small = match &content {
            RegisterContent::Chars(s) => !s.contains('\n'),
            _ => false,
        };

        if is_small {
            self.small_delete = Some(content.clone());
        } else {
            // Shift numbered registers 1-9
            for i in (1..9).rev() {
                self.numbered[i + 1] = self.numbered[i].take();
            }
            self.numbered[1] = Some(content.clone());
        }
        
        self.unnamed = Some(content);
    }

    /// Get content from a register by name
    /// Special registers: ", 0-9, -, a-z
    pub fn get(&self, register: char) -> Option<&RegisterContent> {
        match register {
            '"' => self.unnamed.as_ref(),
            '0'..='9' => {
                let idx = register.to_digit(10).unwrap() as usize;
                self.numbered[idx].as_ref()
            }
            '-' => self.small_delete.as_ref(),
            'a'..='z' | 'A'..='Z' => self.get_named(register),
            '_' => None, // Black hole - always empty
            _ => None,
        }
    }

    /// Set content to a register by name
    pub fn set(&mut self, register: char, content: RegisterContent, is_delete: bool) {
        match register {
            '"' => self.unnamed = Some(content),
            'a'..='z' | 'A'..='Z' => self.set_named(register, content),
            '_' => {} // Black hole - discard
            _ => {
                // For other registers, just set unnamed
                if is_delete {
                    self.delete(content);
                } else {
                    self.yank(content);
                }
            }
        }
    }

    /// Clear all registers
    pub fn clear(&mut self) {
        self.named.clear();
        self.unnamed = None;
        self.small_delete = None;
        self.numbered = Default::default();
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unnamed_register() {
        let mut regs = Registers::new();
        regs.set_unnamed(RegisterContent::Chars("hello".to_string()));
        
        assert_eq!(regs.get_unnamed().unwrap().text(), "hello");
    }

    #[test]
    fn test_yank_to_register_0() {
        let mut regs = Registers::new();
        regs.yank(RegisterContent::Lines("line1\nline2\n".to_string()));
        
        assert!(regs.get('0').is_some());
        assert_eq!(regs.get('0').unwrap().text(), "line1\nline2\n");
    }

    #[test]
    fn test_delete_shifts_registers() {
        let mut regs = Registers::new();
        
        regs.delete(RegisterContent::Lines("first\n".to_string()));
        regs.delete(RegisterContent::Lines("second\n".to_string()));
        regs.delete(RegisterContent::Lines("third\n".to_string()));
        
        assert_eq!(regs.get('1').unwrap().text(), "third\n");
        assert_eq!(regs.get('2').unwrap().text(), "second\n");
        assert_eq!(regs.get('3').unwrap().text(), "first\n");
    }

    #[test]
    fn test_named_register_append() {
        let mut regs = Registers::new();
        
        regs.set_named('a', RegisterContent::Chars("hello".to_string()));
        regs.set_named('A', RegisterContent::Chars(" world".to_string()));
        
        assert_eq!(regs.get_named('a').unwrap().text(), "hello world");
    }

    #[test]
    fn test_small_delete_register() {
        let mut regs = Registers::new();
        
        // Small delete (no newline)
        regs.delete(RegisterContent::Chars("x".to_string()));
        
        assert!(regs.get('-').is_some());
        assert_eq!(regs.get('-').unwrap().text(), "x");
        
        // Numbered registers should NOT be affected
        assert!(regs.get('1').is_none());
    }

    #[test]
    fn test_black_hole_register() {
        let mut regs = Registers::new();
        
        regs.set('_', RegisterContent::Chars("gone".to_string()), false);
        
        // Should be discarded
        assert!(regs.get('_').is_none());
    }

    #[test]
    fn test_linewise_detection() {
        let chars = RegisterContent::Chars("hello".to_string());
        let lines = RegisterContent::Lines("hello\n".to_string());
        
        assert!(!chars.is_linewise());
        assert!(lines.is_linewise());
    }
}
