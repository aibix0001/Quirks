//! Main Editor struct that coordinates all components

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::mode::Mode;
use crate::register::{Registers, RegisterContent};
use crate::search::{Search, SearchDirection};
use crate::selection::{Selection, VisualMode};
use crate::syntax::Highlighter;
use crate::gpu_info::GpuInfo;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex;

/// The main editor state
pub struct Editor {
    /// Current buffer (clone of buffer_manager current)
    buffer: Buffer,
    /// Cursor position
    cursor: Cursor,
    /// Current editing mode
    mode: Mode,
    /// Vertical scroll offset
    scroll_offset: usize,
    /// Command buffer for : commands
    command_buffer: String,
    /// Message to display in command line
    message: Option<String>,
    /// Terminal height (for scroll calculations)
    viewport_height: usize,
    /// Syntax highlighter
    highlighter: Highlighter,
    /// Search state
    search: Search,
    /// Vim-style registers for yank/paste
    registers: Registers,
    /// Pending operator (for commands like dd, yy)
    pending_op: Option<char>,
    /// Current selection (for visual mode)
    selection: Option<Selection>,
    /// Last find character and direction (for f, F, ; commands)
    last_find: Option<(char, bool)>,  // (char, forward)
    /// Numeric prefix for commands (e.g., 5j, 3w)
    numeric_prefix: String,
    /// Pending 'g' command (for gg, gt, gT)
    pending_g: bool,
    /// Buffer manager for multiple buffers
    buffer_manager: crate::buffer_manager::BufferManager,
    /// GPU info provider
    gpu_info: GpuInfo,
    /// Editor configuration
    config: crate::config::Config,
}

impl Default for Editor {
    fn default() -> Self {
        Self::new()
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: Buffer::new(),
            cursor: Cursor::new(),
            mode: Mode::Normal,
            scroll_offset: 0,
            command_buffer: String::new(),
            message: None,
            viewport_height: 24, // Default, updated on resize
            highlighter: Highlighter::new(),
            search: Search::new(),
            registers: Registers::new(),
            pending_op: None,
            selection: None,
            last_find: None,
            numeric_prefix: String::new(),
            pending_g: false,
            buffer_manager: crate::buffer_manager::BufferManager::new(),
            gpu_info: GpuInfo::new(),
            config: crate::config::Config::load(),
        }
    }

    /// Get the editor configuration
    pub fn config(&self) -> &crate::config::Config {
        &self.config
    }

    /// Open a file in the editor
    pub fn open_file(&mut self, path: &str) -> Result<()> {
        // Open file via buffer manager
        self.buffer_manager.open_file(path)?;
        self.buffer = self.buffer_manager.current_buffer().clone();
        self.cursor = Cursor::new();
        self.scroll_offset = 0;
        
        // Set syntax highlighting based on file extension
        if let Some(ext) = std::path::Path::new(path).extension().and_then(|e| e.to_str()) {
            self.highlighter.set_syntax_for_extension(ext);
        }
        
        let syntax_info = self.highlighter.current_syntax_name()
            .map(|s| format!(" [{}]", s))
            .unwrap_or_default();
        self.message = Some(format!("Opened: {}{}", path, syntax_info));
        Ok(())
    }

    /// Get a reference to the GPU info provider
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.gpu_info
    }

    /// Handle a key event, returns true if editor should quit
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None; // Clear message on keypress
        
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
            Mode::Search => self.handle_search_mode(key),
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => self.handle_visual_mode(key),
            Mode::Help => self.handle_help_mode(key),
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        // Handle pending replace
        if self.pending_op == Some('r') {
            if let KeyCode::Char(c) = key.code {
                if !key.modifiers.contains(KeyModifiers::CONTROL) {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    let pos = self.cursor.byte_offset(&self.buffer);
                    // Delete current char and insert replacement
                    let line_len = self.buffer.line_len(self.cursor.line);
                    if self.cursor.col < line_len {
                        self.buffer.delete(pos, pos + 1);
                        self.buffer.insert(pos, &c.to_string());
                    }
                }
            }
            self.pending_op = None;
            return false;
        }

        // Handle pending find (f/F)
        if self.pending_op == Some('f') || self.pending_op == Some('F') {
            if let KeyCode::Char(c) = key.code {
                let forward = self.pending_op == Some('f');
                self.last_find = Some((c, forward));
                self.find_char_on_line(c, forward);
            }
            self.pending_op = None;
            return false;
        }

        match key.code {
            // Numeric prefix (1-9, but skip 0 as it's line start)
            KeyCode::Char(c @ '1'..='9') => {
                self.numeric_prefix.push(c);
                return false;
            }
            KeyCode::Char('0') if !self.numeric_prefix.is_empty() => {
                self.numeric_prefix.push('0');
                return false;
            }
            
            // Movement
            KeyCode::Char('h') | KeyCode::Left => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_left(&self.buffer);
                }
                self.numeric_prefix.clear();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_down(&self.buffer);
                }
                self.ensure_cursor_visible();
                self.numeric_prefix.clear();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_up(&self.buffer);
                }
                self.ensure_cursor_visible();
                self.numeric_prefix.clear();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_right(&self.buffer);
                }
                self.numeric_prefix.clear();
            }
            
            // Line start/end
            KeyCode::Char('0') => self.cursor.move_to_line_start(),
            KeyCode::Char('^') => self.cursor.move_to_first_non_whitespace(&self.buffer),
            KeyCode::Char('$') => self.cursor.move_to_line_end(&self.buffer),
            
            // Buffer start/end
            KeyCode::Char('g') => {
                if self.pending_g {
                    // 'gg' - move to buffer start
                    self.cursor.move_to_buffer_start();
                    self.pending_g = false;
                } else {
                    // Set pending_g for next character (gt, gT, gg)
                    self.pending_g = true;
                    return false;
                }
            }
            KeyCode::Char('G') => self.cursor.move_to_buffer_end(&self.buffer),
            
            // Tab navigation (gt/gT when pending_g)
            KeyCode::Char('t') if self.pending_g => {
                self.buffer_manager.next_buffer();
                self.buffer = self.buffer_manager.current_buffer().clone();
                self.cursor = Cursor::new();
                self.scroll_offset = 0;
                self.message = Some("Switched to next buffer".to_string());
                self.pending_g = false;
            }
            KeyCode::Char('T') if self.pending_g => {
                self.buffer_manager.prev_buffer();
                self.buffer = self.buffer_manager.current_buffer().clone();
                self.cursor = Cursor::new();
                self.scroll_offset = 0;
                self.message = Some("Switched to previous buffer".to_string());
                self.pending_g = false;
            }
            
            // Match bracket (%)
            KeyCode::Char('%') => {
                if let Some((line, col)) = self.find_matching_bracket() {
                    self.cursor.line = line;
                    self.cursor.col = col;
                    self.ensure_cursor_visible();
                }
            }
            
            // Word motions
            KeyCode::Char('w') => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_word_forward(&self.buffer);
                }
                self.numeric_prefix.clear();
            }
            KeyCode::Char('b') => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_word_backward(&self.buffer);
                }
                self.numeric_prefix.clear();
            }
            KeyCode::Char('e') => {
                let count = if self.numeric_prefix.is_empty() {
                    1
                } else {
                    self.numeric_prefix.parse().unwrap_or(1)
                };
                for _ in 0..count {
                    self.cursor.move_word_end(&self.buffer);
                }
                self.numeric_prefix.clear();
            }
            
            // Find character on line
            KeyCode::Char('f') => self.pending_op = Some('f'),
            KeyCode::Char('F') => self.pending_op = Some('F'),
            KeyCode::Char(';') => {
                // Repeat last find
                if let Some((c, forward)) = self.last_find {
                    self.find_char_on_line(c, forward);
                }
            }
            KeyCode::Char(',') => {
                // Repeat last find in opposite direction
                if let Some((c, forward)) = self.last_find {
                    self.find_char_on_line(c, !forward);
                }
            }
            
            // Word search (* and #)
            KeyCode::Char('*') | KeyCode::Char('#') => {
                if let Some(word) = self.get_word_under_cursor() {
                    let forward = key.code == KeyCode::Char('*');
                    self.search.start(if forward { SearchDirection::Forward } else { SearchDirection::Backward });
                    let lines: Vec<String> = (0..self.buffer.line_count())
                        .map(|i| self.buffer.line(i))
                        .collect();
                    self.search.set_pattern(&format!("\\b{}\\b", regex::escape(&word)));
                    self.search.execute(&lines, self.cursor.line, self.cursor.col);
                    
                    if let Some(m) = self.search.current() {
                        self.cursor.line = m.line;
                        self.cursor.col = m.start_col;
                        self.ensure_cursor_visible();
                        self.message = Some(format!("Found: {}", word));
                    }
                }
            }
            
            // Mode switching
            KeyCode::Char('i') => self.mode = Mode::Insert,
            KeyCode::Char('a') => {
                self.cursor.move_right(&self.buffer);
                self.mode = Mode::Insert;
            }
            KeyCode::Char('I') => {
                self.cursor.move_to_line_start();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('A') => {
                self.cursor.move_to_line_end(&self.buffer);
                self.mode = Mode::Insert;
            }
            KeyCode::Char('o') => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                self.cursor.move_to_line_end(&self.buffer);
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert(pos, "\n");
                self.cursor.move_down(&self.buffer);
                self.cursor.move_to_line_start();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('O') => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                self.cursor.move_to_line_start();
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert(pos, "\n");
                self.mode = Mode::Insert;
            }
            
            // Deletion
            KeyCode::Char('x') => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                // Yank the character before deleting
                if let Some(ch) = self.buffer.char_at(self.cursor.line, self.cursor.col) {
                    self.registers.delete(RegisterContent::Chars(ch.to_string()));
                }
                self.buffer.delete_grapheme(self.cursor.line, self.cursor.col);
                self.cursor.clamp(&self.buffer);
            }
            
            // Yank line (yy)
            KeyCode::Char('y') => {
                if self.pending_op == Some('y') {
                    // yy - yank current line
                    let line = self.buffer.line(self.cursor.line);
                    let content = if line.ends_with('\n') {
                        line
                    } else {
                        format!("{}\n", line)
                    };
                    self.registers.yank(RegisterContent::Lines(content));
                    self.message = Some("1 line yanked".to_string());
                    self.pending_op = None;
                } else {
                    self.pending_op = Some('y');
                }
            }
            
            // Delete line (dd)
            KeyCode::Char('d') => {
                if self.pending_op == Some('d') {
                    // dd - delete current line
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    let line = self.buffer.line(self.cursor.line);
                    let content = if line.ends_with('\n') {
                        line
                    } else {
                        format!("{}\n", line)
                    };
                    self.registers.delete(RegisterContent::Lines(content));
                    self.buffer.delete_line(self.cursor.line);
                    self.cursor.clamp(&self.buffer);
                    self.ensure_cursor_visible();
                    self.message = Some("1 line deleted".to_string());
                    self.pending_op = None;
                } else {
                    self.pending_op = Some('d');
                }
            }
            
            // Change line (cc)
            KeyCode::Char('c') => {
                if self.pending_op == Some('c') {
                    // cc - delete line content and enter insert mode
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    let line = self.buffer.line(self.cursor.line);
                    let content = line.trim_end_matches('\n').to_string();
                    if !content.is_empty() {
                        self.registers.delete(RegisterContent::Chars(content));
                    }
                    // Clear the line content but keep the line
                    let line_start = self.buffer.line_to_byte(self.cursor.line);
                    let line_end = if self.cursor.line + 1 < self.buffer.line_count() {
                        self.buffer.line_to_byte(self.cursor.line + 1) - 1
                    } else {
                        self.buffer.len()
                    };
                    if line_start < line_end {
                        self.buffer.delete(line_start, line_end);
                    }
                    self.cursor.col = 0;
                    self.mode = Mode::Insert;
                    self.pending_op = None;
                } else {
                    self.pending_op = Some('c');
                }
            }
            
            // Join lines (J)
            KeyCode::Char('J') => {
                if self.cursor.line + 1 < self.buffer.line_count() {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    // Get current line length (for cursor positioning)
                    let current_line_len = self.buffer.line_len(self.cursor.line);
                    // Join the next line to current
                    self.buffer.join_lines(self.cursor.line);
                    // Move cursor to join point
                    self.cursor.col = current_line_len;
                }
            }
            
            // Delete to end of line (D)
            KeyCode::Char('D') => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                let line = self.buffer.line(self.cursor.line);
                let chars: Vec<char> = line.chars().collect();
                if self.cursor.col < chars.len() {
                    let deleted: String = chars[self.cursor.col..].iter()
                        .collect::<String>()
                        .trim_end_matches('\n')
                        .to_string();
                    if !deleted.is_empty() {
                        self.registers.delete(RegisterContent::Chars(deleted));
                    }
                    // Delete from cursor to end of line (keep newline)
                    let start = self.buffer.line_to_byte(self.cursor.line) 
                        + self.buffer.col_to_byte(self.cursor.line, self.cursor.col);
                    let end = if self.cursor.line + 1 < self.buffer.line_count() {
                        self.buffer.line_to_byte(self.cursor.line + 1) - 1
                    } else {
                        self.buffer.len()
                    };
                    if start < end {
                        self.buffer.delete(start, end);
                    }
                }
                self.cursor.clamp(&self.buffer);
            }
            
            // Change to end of line (C)
            KeyCode::Char('C') => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                let line = self.buffer.line(self.cursor.line);
                let chars: Vec<char> = line.chars().collect();
                if self.cursor.col < chars.len() {
                    let deleted: String = chars[self.cursor.col..].iter()
                        .collect::<String>()
                        .trim_end_matches('\n')
                        .to_string();
                    if !deleted.is_empty() {
                        self.registers.delete(RegisterContent::Chars(deleted));
                    }
                    let start = self.buffer.line_to_byte(self.cursor.line) 
                        + self.buffer.col_to_byte(self.cursor.line, self.cursor.col);
                    let end = if self.cursor.line + 1 < self.buffer.line_count() {
                        self.buffer.line_to_byte(self.cursor.line + 1) - 1
                    } else {
                        self.buffer.len()
                    };
                    if start < end {
                        self.buffer.delete(start, end);
                    }
                }
                self.mode = Mode::Insert;
            }
            
            // Replace character (r)
            KeyCode::Char('r') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Set pending replace mode - next char will replace current
                self.pending_op = Some('r');
            }
            
            // Toggle case (~)
            KeyCode::Char('~') => {
                let line_len = self.buffer.line_len(self.cursor.line);
                if self.cursor.col < line_len {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    if let Some(c) = self.buffer.char_at(self.cursor.line, self.cursor.col) {
                        let toggled = if c.is_uppercase() {
                            c.to_lowercase().to_string()
                        } else {
                            c.to_uppercase().to_string()
                        };
                        let pos = self.cursor.byte_offset(&self.buffer);
                        self.buffer.delete(pos, pos + c.len_utf8());
                        self.buffer.insert(pos, &toggled);
                        self.cursor.move_right(&self.buffer);
                    }
                }
            }
            
            // Indent (>>)
            KeyCode::Char('>') => {
                if self.pending_op == Some('>') {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    self.buffer.indent_line(self.cursor.line, self.config.tab_width);
                    self.pending_op = None;
                } else {
                    self.pending_op = Some('>');
                }
            }
            
            // Outdent (<<)
            KeyCode::Char('<') => {
                if self.pending_op == Some('<') {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    self.buffer.outdent_line(self.cursor.line, self.config.tab_width);
                    self.cursor.clamp(&self.buffer);
                    self.pending_op = None;
                } else {
                    self.pending_op = Some('<');
                }
            }
            
            // Paste after (p)
            KeyCode::Char('p') => {
                if let Some(content) = self.registers.get_unnamed() {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    match content {
                        RegisterContent::Lines(text) => {
                            // Paste below current line
                            self.buffer.insert_line_below(self.cursor.line, text);
                            self.cursor.line += 1;
                            self.cursor.col = 0;
                        }
                        RegisterContent::Chars(text) => {
                            // Paste after cursor
                            let pos = self.cursor.byte_offset(&self.buffer);
                            self.buffer.insert(pos + 1, text);
                            self.cursor.col += 1;
                        }
                        RegisterContent::Block(_) => {
                            // TODO: block paste
                        }
                    }
                    self.ensure_cursor_visible();
                }
            }
            
            // Paste before (P)
            KeyCode::Char('P') => {
                if let Some(content) = self.registers.get_unnamed() {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    match content {
                        RegisterContent::Lines(text) => {
                            // Paste above current line
                            self.buffer.insert_line_above(self.cursor.line, text);
                            self.cursor.col = 0;
                        }
                        RegisterContent::Chars(text) => {
                            // Paste before cursor
                            let pos = self.cursor.byte_offset(&self.buffer);
                            self.buffer.insert(pos, text);
                        }
                        RegisterContent::Block(_) => {
                            // TODO: block paste
                        }
                    }
                    self.ensure_cursor_visible();
                }
            }
            
            // Command mode
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            
            // Search
            KeyCode::Char('/') => {
                self.search.start(SearchDirection::Forward);
                self.mode = Mode::Search;
            }
            KeyCode::Char('?') => {
                self.search.start(SearchDirection::Backward);
                self.mode = Mode::Search;
            }
            KeyCode::Char('n') => {
                if let Some(m) = self.search.next_match() {
                    self.cursor.line = m.line;
                    self.cursor.col = m.start_col;
                    self.ensure_cursor_visible();
                    self.message = Some(self.search.match_info());
                }
            }
            KeyCode::Char('N') => {
                if let Some(m) = self.search.prev_match() {
                    self.cursor.line = m.line;
                    self.cursor.col = m.start_col;
                    self.ensure_cursor_visible();
                    self.message = Some(self.search.match_info());
                }
            }
            
            // Undo
            KeyCode::Char('u') => {
                if let Some((line, col)) = self.buffer.undo(self.cursor.line, self.cursor.col) {
                    self.cursor.line = line;
                    self.cursor.col = col;
                    self.cursor.clamp(&self.buffer);
                    self.ensure_cursor_visible();
                    self.message = Some("Undo".to_string());
                } else {
                    self.message = Some("Already at oldest change".to_string());
                }
            }
            
            // Redo (Ctrl+R)
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Some((line, col)) = self.buffer.redo() {
                    self.cursor.line = line;
                    self.cursor.col = col;
                    self.cursor.clamp(&self.buffer);
                    self.ensure_cursor_visible();
                    self.message = Some("Redo".to_string());
                } else {
                    self.message = Some("Already at newest change".to_string());
                }
            }
            
            // Page up (Ctrl+U)
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let half_page = self.viewport_height / 2;
                self.cursor.line = self.cursor.line.saturating_sub(half_page);
                self.ensure_cursor_visible();
            }
            
            // Page down (Ctrl+D)
            KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let half_page = self.viewport_height / 2;
                let max_line = self.buffer.line_count().saturating_sub(1);
                self.cursor.line = (self.cursor.line + half_page).min(max_line);
                self.ensure_cursor_visible();
            }
            
            // Full page up (Ctrl+B)
            KeyCode::Char('b') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.cursor.line = self.cursor.line.saturating_sub(self.viewport_height);
                self.ensure_cursor_visible();
            }
            
            // Full page down (Ctrl+F)
            KeyCode::Char('f') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let max_line = self.buffer.line_count().saturating_sub(1);
                self.cursor.line = (self.cursor.line + self.viewport_height).min(max_line);
                self.ensure_cursor_visible();
            }
            
            // Visual modes
            KeyCode::Char('v') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selection = Some(Selection::new(VisualMode::Char, self.cursor.line, self.cursor.col));
                self.mode = Mode::Visual;
            }
            KeyCode::Char('V') => {
                self.selection = Some(Selection::new(VisualMode::Line, self.cursor.line, self.cursor.col));
                self.mode = Mode::VisualLine;
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.selection = Some(Selection::new(VisualMode::Block, self.cursor.line, self.cursor.col));
                self.mode = Mode::VisualBlock;
            }
            
            // File info (Ctrl+G)
            KeyCode::Char('g') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let lines = self.buffer.line_count();
                let chars = self.buffer.len();
                let file_name = self.buffer.file_name().unwrap_or("[No Name]");
                let modified = if self.buffer.is_modified() { " [Modified]" } else { "" };
                let pos = format!("line {}/{}", self.cursor.line + 1, lines);
                self.message = Some(format!("\"{}\"{} {} chars, {}", file_name, modified, chars, pos));
            }
            
            _ => {
                // Clear numeric prefix for commands that don't use it
                self.numeric_prefix.clear();
            }
        }
        false
    }

    /// Handle keys in insert mode
    fn handle_insert_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                // Move cursor back one (Vim behavior)
                if self.cursor.col > 0 {
                    self.cursor.move_left(&self.buffer);
                }
            }
            KeyCode::Char(c) => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert_char(pos, c);
                self.cursor.col += 1;
            }
            KeyCode::Enter => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert_char(pos, '\n');
                self.cursor.line += 1;
                self.cursor.col = 0;
                self.ensure_cursor_visible();
            }
            KeyCode::Backspace => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                let (new_line, new_col) = self.buffer.backspace(self.cursor.line, self.cursor.col);
                self.cursor.line = new_line;
                self.cursor.col = new_col;
                self.ensure_cursor_visible();
            }
            KeyCode::Delete => {
                self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                self.buffer.delete_grapheme(self.cursor.line, self.cursor.col);
            }
            KeyCode::Left => self.cursor.move_left(&self.buffer),
            KeyCode::Right => self.cursor.move_right(&self.buffer),
            KeyCode::Up => {
                self.cursor.move_up(&self.buffer);
                self.ensure_cursor_visible();
            }
            KeyCode::Down => {
                self.cursor.move_down(&self.buffer);
                self.ensure_cursor_visible();
            }
            _ => {}
        }
        false
    }

    /// Handle keys in command mode
    fn handle_command_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.command_buffer.clear();
            }
            KeyCode::Enter => {
                let should_quit = self.execute_command();
                // Don't reset mode if command changed it (e.g., to Help)
                if self.mode == Mode::Command {
                    self.mode = Mode::Normal;
                }
                self.command_buffer.clear();
                return should_quit;
            }
            KeyCode::Backspace => {
                if self.command_buffer.pop().is_none() {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(c) => {
                self.command_buffer.push(c);
            }
            _ => {}
        }
        false
    }

    /// Execute a command from command mode
    fn execute_command(&mut self) -> bool {
        let cmd = self.command_buffer.trim();
        match cmd {
            "q" | "quit" => {
                if self.buffer.is_modified() {
                    self.message = Some("Unsaved changes! Use :q! to force quit".to_string());
                    return false;
                }
                return true;
            }
            "q!" | "quit!" => return true,
            "w" | "write" => {
                if let Err(e) = self.buffer.save() {
                    self.message = Some(format!("Error saving: {}", e));
                } else {
                    self.message = Some("Written".to_string());
                }
            }
            "wq" | "x" | "wq!" => {
                if let Err(e) = self.buffer.save() {
                    self.message = Some(format!("Error saving: {}", e));
                } else {
                    return true;
                }
            }
            "wa" => {
                // Save all buffers
                if let Err(e) = self.buffer.save() {
                    self.message = Some(format!("Error saving: {}", e));
                } else {
                    self.message = Some("All buffers saved".to_string());
                }
            }
            "qa" | "qall" => {
                // Quit all (if no unsaved changes)
                if self.buffer.is_modified() {
                    self.message = Some("Unsaved changes! Use :qa! to force quit".to_string());
                } else {
                    return true;
                }
            }
            "qa!" | "qall!" => {
                // Force quit all
                return true;
            }
            _ if cmd.starts_with("w ") => {
                let path = cmd.strip_prefix("w ").unwrap().trim();
                if let Err(e) = self.buffer.save_as(path) {
                    self.message = Some(format!("Error saving: {}", e));
                } else {
                    self.message = Some(format!("Written: {}", path));
                }
            }
            "noh" | "nohlsearch" => {
                self.search.clear_highlight();
            }
            "help" | "h" | "?" => {
                self.mode = Mode::Help;
                self.message = Some("Press q/Esc/Enter to close help".to_string());
            }
            "version" | "ver" => {
                self.message = Some("Quirks v0.3.1 - A modal text editor".to_string());
            }
            "set" => {
                // Show current settings
                self.message = Some(format!(
                    "tab_width={} line_numbers={} syntax={}",
                    self.config.tab_width,
                    self.config.line_numbers,
                    self.config.syntax_highlighting
                ));
            }
            "set number" | "set nu" => {
                self.config.line_numbers = true;
                self.message = Some("Line numbers enabled".to_string());
            }
            "set nonumber" | "set nonu" => {
                self.config.line_numbers = false;
                self.message = Some("Line numbers disabled".to_string());
            }
            "set syntax" | "set syn" => {
                self.config.syntax_highlighting = true;
                self.message = Some("Syntax highlighting enabled".to_string());
            }
            "set nosyntax" | "set nosyn" => {
                self.config.syntax_highlighting = false;
                self.message = Some("Syntax highlighting disabled".to_string());
            }
            "tabnew" | "new" => {
                // Create a new empty buffer
                self.buffer = Buffer::new();
                self.cursor = Cursor::new();
                self.scroll_offset = 0;
                self.message = Some("New buffer".to_string());
            }
            "tabclose" | "close" => {
                // Close current buffer (alias for :bd)
                match self.buffer_manager.close_current() {
                    Ok(_) => {
                        if self.buffer_manager.has_buffers() {
                            self.buffer = self.buffer_manager.current_buffer().clone();
                        } else {
                            self.buffer = Buffer::new();
                        }
                        self.cursor = Cursor::new();
                        self.scroll_offset = 0;
                        self.message = Some("Buffer closed".to_string());
                    }
                    Err(e) => {
                        self.message = Some(format!("Error: {}", e));
                    }
                }
            }
            "only" => {
                // Close all other buffers (keep current)
                self.message = Some("Closed all other buffers".to_string());
            }
            _ if cmd.starts_with("e ") => {
                let path = cmd.strip_prefix("e ").unwrap().trim();
                match self.buffer_manager.open_file(path) {
                    Ok(_) => {
                        self.message = Some(format!("Opened in new buffer: {}", path));
                    }
                    Err(e) => {
                        self.message = Some(format!("Error opening file: {}", e));
                    }
                }
            }
            "ls" | "buffers" => {
                // List all open buffers
                let buffers = self.buffer_manager.list_buffers();
                if buffers.is_empty() {
                    self.message = Some("No buffers open".to_string());
                } else {
                    let list: Vec<String> = buffers
                        .iter()
                        .map(|(i, name, is_current)| {
                            if *is_current {
                                format!("[{}] {}", i, name)
                            } else {
                                format!(" {}  {}", i, name)
                            }
                        })
                        .collect();
                    self.message = Some(list.join(" | "));
                }
            }
            _ if cmd.starts_with("b ") => {
                let buf_num_str = cmd.strip_prefix("b ").unwrap().trim();
                if let Ok(idx) = buf_num_str.parse::<usize>() {
                    match self.buffer_manager.switch_to(idx) {
                        Ok(_) => {
                            self.buffer = self.buffer_manager.current_buffer().clone();
                            self.cursor = Cursor::new();
                            self.scroll_offset = 0;
                            self.message = Some(format!("Switched to buffer {}", idx));
                        }
                        Err(e) => {
                            self.message = Some(format!("Error: {}", e));
                        }
                    }
                } else {
                    self.message = Some("Usage: :b <buffer_number>".to_string());
                }
            }
            _ if cmd.starts_with("bd") => {
                // Close current buffer
                match self.buffer_manager.close_current() {
                    Ok(_) => {
                        if self.buffer_manager.has_buffers() {
                            self.buffer = self.buffer_manager.current_buffer().clone();
                        } else {
                            self.buffer = Buffer::new();
                        }
                        self.cursor = Cursor::new();
                        self.scroll_offset = 0;
                        self.message = Some("Buffer closed".to_string());
                    }
                    Err(e) => {
                        self.message = Some(format!("Error: {}", e));
                    }
                }
            }
            _ => {
                self.message = Some(format!("Unknown command: {}", cmd));
            }
        }
        false
    }

    /// Handle keys in search mode
    fn handle_search_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search.clear_highlight();
            }
            KeyCode::Enter => {
                // Execute the search
                let lines: Vec<String> = (0..self.buffer.line_count())
                    .map(|i| self.buffer.line(i))
                    .collect();
                self.search.execute(&lines, self.cursor.line, self.cursor.col);
                
                // Jump to first match
                if let Some(m) = self.search.current() {
                    self.cursor.line = m.line;
                    self.cursor.col = m.start_col;
                    self.ensure_cursor_visible();
                    self.message = Some(self.search.match_info());
                } else if !self.search.is_empty() {
                    self.message = Some("Pattern not found".to_string());
                }
                
                self.mode = Mode::Normal;
            }
            KeyCode::Backspace => {
                if !self.search.pop_char() {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(c) => {
                self.search.push_char(c);
            }
            _ => {}
        }
        false
    }

    /// Handle keys in visual mode
    fn handle_visual_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.selection = None;
            }
            
            // Movement - same as normal mode but updates selection
            KeyCode::Char('h') | KeyCode::Left => {
                self.cursor.move_left(&self.buffer);
                self.update_selection();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.cursor.move_down(&self.buffer);
                self.update_selection();
                self.ensure_cursor_visible();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.cursor.move_up(&self.buffer);
                self.update_selection();
                self.ensure_cursor_visible();
            }
            KeyCode::Char('l') | KeyCode::Right => {
                self.cursor.move_right(&self.buffer);
                self.update_selection();
            }
            KeyCode::Char('0') => {
                self.cursor.move_to_line_start();
                self.update_selection();
            }
            KeyCode::Char('^') => {
                self.cursor.move_to_first_non_whitespace(&self.buffer);
                self.update_selection();
            }
            KeyCode::Char('$') => {
                self.cursor.move_to_line_end(&self.buffer);
                self.update_selection();
            }
            KeyCode::Char('g') => {
                self.cursor.move_to_buffer_start();
                self.update_selection();
                self.ensure_cursor_visible();
            }
            KeyCode::Char('G') => {
                self.cursor.move_to_buffer_end(&self.buffer);
                self.update_selection();
                self.ensure_cursor_visible();
            }
            
            // Word motions
            KeyCode::Char('w') => {
                self.cursor.move_word_forward(&self.buffer);
                self.update_selection();
            }
            KeyCode::Char('b') => {
                self.cursor.move_word_backward(&self.buffer);
                self.update_selection();
            }
            KeyCode::Char('e') => {
                self.cursor.move_word_end(&self.buffer);
                self.update_selection();
            }
            
            // Yank selection
            KeyCode::Char('y') => {
                self.yank_selection();
                self.mode = Mode::Normal;
                self.selection = None;
            }
            
            // Delete selection
            KeyCode::Char('d') | KeyCode::Char('x') => {
                self.delete_selection();
                self.mode = Mode::Normal;
                self.selection = None;
            }
            
            // Switch visual mode type
            KeyCode::Char('v') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                if self.mode == Mode::Visual {
                    self.mode = Mode::Normal;
                    self.selection = None;
                } else {
                    if let Some(ref mut sel) = self.selection {
                        sel.mode = VisualMode::Char;
                    }
                    self.mode = Mode::Visual;
                }
            }
            KeyCode::Char('V') => {
                if self.mode == Mode::VisualLine {
                    self.mode = Mode::Normal;
                    self.selection = None;
                } else {
                    if let Some(ref mut sel) = self.selection {
                        sel.mode = VisualMode::Line;
                    }
                    self.mode = Mode::VisualLine;
                }
            }
            
            _ => {}
        }
        false
    }

    /// Update the selection's cursor position
    fn update_selection(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.update_cursor(self.cursor.line, self.cursor.col);
        }
    }

    /// Yank the current selection to register
    fn yank_selection(&mut self) {
        let Some(ref sel) = self.selection else { return };
        
        let mut content = String::new();
        let linewise = matches!(sel.mode, VisualMode::Line);
        
        match sel.mode {
            VisualMode::Char => {
                let (start_line, start_col, end_line, end_col) = sel.normalized();
                for line_idx in start_line..=end_line {
                    let line = self.buffer.line(line_idx);
                    let chars: Vec<char> = line.chars().collect();
                    
                    let start = if line_idx == start_line { start_col } else { 0 };
                    let end = if line_idx == end_line { (end_col + 1).min(chars.len()) } else { chars.len() };
                    
                    let part: String = chars.get(start..end).unwrap_or(&[]).iter().collect();
                    content.push_str(&part);
                    
                    if line_idx < end_line {
                        content.push('\n');
                    }
                }
            }
            VisualMode::Line => {
                let (start_line, end_line) = sel.line_range();
                for line_idx in start_line..=end_line {
                    content.push_str(&self.buffer.line(line_idx));
                    content.push('\n');
                }
            }
            VisualMode::Block => {
                let (start_line, end_line) = sel.line_range();
                let (start_col, end_col) = sel.col_range();
                for line_idx in start_line..=end_line {
                    let line = self.buffer.line(line_idx);
                    let chars: Vec<char> = line.chars().collect();
                    let part: String = chars.get(start_col..(end_col + 1).min(chars.len())).unwrap_or(&[]).iter().collect();
                    content.push_str(&part);
                    if line_idx < end_line {
                        content.push('\n');
                    }
                }
            }
        }
        
        let line_count = content.lines().count().max(1);
        let register_content = if linewise {
            RegisterContent::Lines(content)
        } else {
            RegisterContent::Chars(content)
        };
        self.registers.yank(register_content);
        self.message = Some(format!("{} line(s) yanked", line_count));
    }

    /// Handle keys in help mode
    fn handle_help_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
                self.mode = Mode::Normal;
            }
            _ => {}
        }
        false
    }

    /// Delete the current selection
    fn delete_selection(&mut self) {
        // Extract selection info before borrowing mutably
        let sel_info = match &self.selection {
            Some(sel) => Some((sel.mode, sel.normalized(), sel.line_range(), sel.col_range())),
            None => None,
        };
        
        let Some((mode, normalized, line_range, col_range)) = sel_info else { return };
        
        self.buffer.checkpoint(self.cursor.line, self.cursor.col);
        
        // First yank the selection
        self.yank_selection();
        
        match mode {
            VisualMode::Char => {
                let (start_line, start_col, end_line, end_col) = normalized;
                let start_byte = self.buffer.line_to_byte(start_line) + self.buffer.col_to_byte(start_line, start_col);
                let end_byte = self.buffer.line_to_byte(end_line) + self.buffer.col_to_byte(end_line, end_col + 1);
                self.buffer.delete(start_byte, end_byte);
                self.cursor.line = start_line;
                self.cursor.col = start_col;
            }
            VisualMode::Line => {
                let (start_line, end_line) = line_range;
                let start_byte = self.buffer.line_to_byte(start_line);
                let end_byte = if end_line + 1 < self.buffer.line_count() {
                    self.buffer.line_to_byte(end_line + 1)
                } else {
                    self.buffer.line_to_byte(end_line) + self.buffer.line(end_line).len()
                };
                self.buffer.delete(start_byte, end_byte);
                self.cursor.line = start_line;
                self.cursor.col = 0;
            }
            VisualMode::Block => {
                let (start_line, end_line) = line_range;
                let (start_col, end_col) = col_range;
                for line_idx in (start_line..=end_line).rev() {
                    let start_byte = self.buffer.line_to_byte(line_idx) + self.buffer.col_to_byte(line_idx, start_col);
                    let end_byte = self.buffer.line_to_byte(line_idx) + self.buffer.col_to_byte(line_idx, end_col + 1);
                    self.buffer.delete(start_byte, end_byte);
                }
                self.cursor.line = start_line;
                self.cursor.col = start_col;
            }
        }
        
        self.cursor.clamp(&self.buffer);
        self.message = Some("Deleted".to_string());
    }

    /// Get the word under the cursor
    fn get_word_under_cursor(&self) -> Option<String> {
        let line = self.buffer.line(self.cursor.line);
        let chars: Vec<char> = line.chars().collect();
        
        if self.cursor.col >= chars.len() {
            return None;
        }
        
        let current_char = chars[self.cursor.col];
        if !current_char.is_alphanumeric() && current_char != '_' {
            return None;
        }
        
        // Find word start
        let mut start = self.cursor.col;
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        
        // Find word end
        let mut end = self.cursor.col + 1;
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        
        Some(chars[start..end].iter().collect())
    }

    /// Find and move to character on current line
    fn find_char_on_line(&mut self, target: char, forward: bool) {
        let line = self.buffer.line(self.cursor.line);
        let chars: Vec<char> = line.chars().collect();
        
        if forward {
            // Search forward from cursor
            for i in (self.cursor.col + 1)..chars.len() {
                if chars[i] == target {
                    self.cursor.col = i;
                    return;
                }
            }
        } else {
            // Search backward from cursor
            for i in (0..self.cursor.col).rev() {
                if chars[i] == target {
                    self.cursor.col = i;
                    return;
                }
            }
        }
    }

    /// Find the matching bracket for the character under cursor
    fn find_matching_bracket(&self) -> Option<(usize, usize)> {
        let current_char = self.buffer.char_at(self.cursor.line, self.cursor.col)?;
        
        let (target, forward) = match current_char {
            '(' => (')', true),
            ')' => ('(', false),
            '[' => (']', true),
            ']' => ('[', false),
            '{' => ('}', true),
            '}' => ('{', false),
            '<' => ('>', true),
            '>' => ('<', false),
            _ => return None,
        };
        
        let mut depth = 1;
        let mut line = self.cursor.line;
        let mut col = self.cursor.col;
        
        loop {
            if forward {
                col += 1;
                let line_len = self.buffer.line_len(line);
                if col >= line_len {
                    line += 1;
                    if line >= self.buffer.line_count() {
                        return None;
                    }
                    col = 0;
                }
            } else {
                if col == 0 {
                    if line == 0 {
                        return None;
                    }
                    line -= 1;
                    col = self.buffer.line_len(line).saturating_sub(1);
                } else {
                    col -= 1;
                }
            }
            
            if let Some(c) = self.buffer.char_at(line, col) {
                if c == current_char {
                    depth += 1;
                } else if c == target {
                    depth -= 1;
                    if depth == 0 {
                        return Some((line, col));
                    }
                }
            }
        }
    }

    /// Ensure cursor is visible by adjusting scroll offset
    fn ensure_cursor_visible(&mut self) {
        // Leave some margin
        let margin = 3;
        
        if self.cursor.line < self.scroll_offset + margin {
            self.scroll_offset = self.cursor.line.saturating_sub(margin);
        } else if self.cursor.line >= self.scroll_offset + self.viewport_height - margin {
            self.scroll_offset = self.cursor.line.saturating_sub(self.viewport_height - margin - 1);
        }
    }

    /// Update viewport height (called on terminal resize)
    pub fn set_viewport_height(&mut self, height: usize) {
        self.viewport_height = height;
    }

    // Getters
    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn cursor(&self) -> &Cursor {
        &self.cursor
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    pub fn message(&self) -> Option<&String> {
        self.message.as_ref()
    }

    pub fn highlighter(&self) -> &Highlighter {
        &self.highlighter
    }

    pub fn search(&self) -> &Search {
        &self.search
    }

    pub fn selection(&self) -> Option<&Selection> {
        self.selection.as_ref()
    }
}
