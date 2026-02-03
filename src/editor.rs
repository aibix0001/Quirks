//! Main Editor struct that coordinates all components

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::mode::Mode;
use crate::register::{Registers, RegisterContent};
use crate::search::{Search, SearchDirection};
use crate::selection::{Selection, VisualMode};
use crate::syntax::Highlighter;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The main editor state
pub struct Editor {
    /// Current buffer
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
        }
    }

    /// Open a file in the editor
    pub fn open_file(&mut self, path: &str) -> Result<()> {
        self.buffer = Buffer::from_file(path)?;
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

    /// Handle a key event, returns true if editor should quit
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        self.message = None; // Clear message on keypress
        
        match self.mode {
            Mode::Normal => self.handle_normal_mode(key),
            Mode::Insert => self.handle_insert_mode(key),
            Mode::Command => self.handle_command_mode(key),
            Mode::Search => self.handle_search_mode(key),
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => self.handle_visual_mode(key),
        }
    }

    /// Handle keys in normal mode
    fn handle_normal_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            // Movement
            KeyCode::Char('h') | KeyCode::Left => self.cursor.move_left(&self.buffer),
            KeyCode::Char('j') | KeyCode::Down => {
                self.cursor.move_down(&self.buffer);
                self.ensure_cursor_visible();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.cursor.move_up(&self.buffer);
                self.ensure_cursor_visible();
            }
            KeyCode::Char('l') | KeyCode::Right => self.cursor.move_right(&self.buffer),
            
            // Line start/end
            KeyCode::Char('0') => self.cursor.move_to_line_start(),
            KeyCode::Char('$') => self.cursor.move_to_line_end(&self.buffer),
            
            // Buffer start/end
            KeyCode::Char('g') => self.cursor.move_to_buffer_start(),
            KeyCode::Char('G') => self.cursor.move_to_buffer_end(&self.buffer),
            
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
            
            // Visual mode
            KeyCode::Char('v') => {
                self.mode = Mode::Visual;
                self.selection = Some(Selection::new(
                    VisualMode::Char,
                    self.cursor.line,
                    self.cursor.col,
                ));
            }
            KeyCode::Char('V') => {
                self.mode = Mode::VisualLine;
                self.selection = Some(Selection::new(
                    VisualMode::Line,
                    self.cursor.line,
                    self.cursor.col,
                ));
            }
            KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.mode = Mode::VisualBlock;
                self.selection = Some(Selection::new(
                    VisualMode::Block,
                    self.cursor.line,
                    self.cursor.col,
                ));
            }
            
            _ => {}
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
                self.mode = Mode::Normal;
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
            "wq" | "x" => {
                if let Err(e) = self.buffer.save() {
                    self.message = Some(format!("Error saving: {}", e));
                } else {
                    return true;
                }
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
            // Exit visual mode
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.selection = None;
            }
            
            // Movement (same as normal mode, but updates selection)
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
            
            // Line start/end
            KeyCode::Char('0') => {
                self.cursor.move_to_line_start();
                self.update_selection();
            }
            KeyCode::Char('$') => {
                self.cursor.move_to_line_end(&self.buffer);
                self.update_selection();
            }
            
            // Buffer start/end
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
            
            // Yank selection
            KeyCode::Char('y') => {
                if let Some(text) = self.get_selected_text() {
                    let content = if self.mode == Mode::VisualLine {
                        RegisterContent::Lines(text)
                    } else {
                        RegisterContent::Chars(text)
                    };
                    self.registers.yank(content);
                    self.message = Some("Yanked".to_string());
                }
                self.mode = Mode::Normal;
                self.selection = None;
            }
            
            // Delete selection
            KeyCode::Char('d') | KeyCode::Char('x') => {
                if let Some(text) = self.get_selected_text() {
                    self.buffer.checkpoint(self.cursor.line, self.cursor.col);
                    let content = if self.mode == Mode::VisualLine {
                        RegisterContent::Lines(text)
                    } else {
                        RegisterContent::Chars(text)
                    };
                    self.registers.delete(content);
                    self.delete_selection();
                    self.message = Some("Deleted".to_string());
                }
                self.mode = Mode::Normal;
                self.selection = None;
            }
            
            // Switch visual modes
            KeyCode::Char('v') => {
                if self.mode == Mode::Visual {
                    self.mode = Mode::Normal;
                    self.selection = None;
                } else {
                    self.mode = Mode::Visual;
                    if let Some(ref mut sel) = self.selection {
                        sel.mode = VisualMode::Char;
                    }
                }
            }
            KeyCode::Char('V') => {
                if self.mode == Mode::VisualLine {
                    self.mode = Mode::Normal;
                    self.selection = None;
                } else {
                    self.mode = Mode::VisualLine;
                    if let Some(ref mut sel) = self.selection {
                        sel.mode = VisualMode::Line;
                    }
                }
            }
            
            _ => {}
        }
        false
    }

    /// Update the selection endpoint to current cursor position
    fn update_selection(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.update_cursor(self.cursor.line, self.cursor.col);
        }
    }

    /// Get the text covered by the current selection
    fn get_selected_text(&self) -> Option<String> {
        let sel = self.selection.as_ref()?;
        let (start_line, start_col, end_line, end_col) = sel.normalized();
        
        match sel.mode {
            VisualMode::Char => {
                let mut text = String::new();
                for line_idx in start_line..=end_line {
                    let line = self.buffer.line(line_idx);
                    if start_line == end_line {
                        // Single line selection
                        let chars: Vec<char> = line.chars().collect();
                        let end: usize = (end_col + 1).min(chars.len());
                        text.extend(&chars[start_col..end]);
                    } else if line_idx == start_line {
                        let chars: Vec<char> = line.chars().collect();
                        text.extend(&chars[start_col..]);
                    } else if line_idx == end_line {
                        let chars: Vec<char> = line.chars().collect();
                        let end: usize = (end_col + 1).min(chars.len());
                        text.extend(&chars[..end]);
                    } else {
                        text.push_str(&line);
                    }
                }
                Some(text)
            }
            VisualMode::Line => {
                let mut text = String::new();
                for line_idx in start_line..=end_line {
                    let line = self.buffer.line(line_idx);
                    text.push_str(&line);
                    if !line.ends_with('\n') {
                        text.push('\n');
                    }
                }
                Some(text)
            }
            VisualMode::Block => {
                // TODO: Implement block selection text extraction
                None
            }
        }
    }

    /// Delete the text covered by the current selection
    fn delete_selection(&mut self) {
        let sel: &Selection = match self.selection.as_ref() {
            Some(s) => s,
            None => return,
        };
        let (start_line, start_col, end_line, end_col) = sel.normalized();
        
        match sel.mode {
            VisualMode::Line => {
                // Delete entire lines
                for _ in start_line..=end_line {
                    self.buffer.delete_line(start_line);
                }
                self.cursor.line = start_line.min(self.buffer.line_count().saturating_sub(1));
                self.cursor.col = 0;
            }
            VisualMode::Char => {
                // Character-wise deletion
                if start_line == end_line {
                    // Single line
                    for _ in start_col..=end_col {
                        self.buffer.delete_grapheme(start_line, start_col);
                    }
                    self.cursor.line = start_line;
                    self.cursor.col = start_col;
                } else {
                    // Multi-line - delete from end to start to preserve positions
                    // Delete end line portion
                    let end_line_content = self.buffer.line(end_line);
                    let end_chars: Vec<char> = end_line_content.chars().collect();
                    let remaining_end: String = end_chars.iter().skip(end_col + 1).collect();
                    
                    // Delete middle lines (from end to start)
                    for line_idx in (start_line + 1..=end_line).rev() {
                        self.buffer.delete_line(line_idx);
                    }
                    
                    // Truncate start line and append remaining end
                    let start_line_content = self.buffer.line(start_line);
                    let start_chars: Vec<char> = start_line_content.chars().collect();
                    let new_line: String = start_chars.iter().take(start_col).collect::<String>() + &remaining_end;
                    
                    self.buffer.delete_line(start_line);
                    self.buffer.insert_line_above(start_line.min(self.buffer.line_count()), &new_line);
                    
                    self.cursor.line = start_line;
                    self.cursor.col = start_col;
                }
            }
            VisualMode::Block => {
                // TODO: Implement block deletion
            }
        }
        
        self.cursor.clamp(&self.buffer);
        self.ensure_cursor_visible();
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
