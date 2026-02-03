//! Main Editor struct that coordinates all components

use crate::buffer::Buffer;
use crate::cursor::Cursor;
use crate::mode::Mode;
use crate::search::{Search, SearchDirection};
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
    /// Search buffer for / and ? commands
    search_buffer: String,
    /// Search state
    search: Search,
    /// Message to display in command line
    message: Option<String>,
    /// Terminal height (for scroll calculations)
    viewport_height: usize,
    /// Syntax highlighter
    highlighter: Highlighter,
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
            search_buffer: String::new(),
            search: Search::new(),
            message: None,
            viewport_height: 24, // Default, updated on resize
            highlighter: Highlighter::new(),
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
            Mode::SearchForward | Mode::SearchBackward => self.handle_search_mode(key),
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
                self.cursor.move_to_line_end(&self.buffer);
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert(pos, "\n");
                self.cursor.move_down(&self.buffer);
                self.cursor.move_to_line_start();
                self.mode = Mode::Insert;
            }
            KeyCode::Char('O') => {
                self.cursor.move_to_line_start();
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert(pos, "\n");
                self.mode = Mode::Insert;
            }
            
            // Deletion
            KeyCode::Char('x') => {
                self.buffer.delete_grapheme(self.cursor.line, self.cursor.col);
                self.cursor.clamp(&self.buffer);
            }
            
            // Command mode
            KeyCode::Char(':') => {
                self.mode = Mode::Command;
                self.command_buffer.clear();
            }
            
            // Search
            KeyCode::Char('/') => {
                self.mode = Mode::SearchForward;
                self.search_buffer.clear();
            }
            KeyCode::Char('?') => {
                self.mode = Mode::SearchBackward;
                self.search_buffer.clear();
            }
            KeyCode::Char('n') => {
                // Next match
                if let Some(m) = self.search.jump_next(&self.cursor) {
                    self.cursor.line = m.line;
                    self.cursor.col = m.start_col;
                    self.ensure_cursor_visible();
                    let idx = self.search.current_match_index().unwrap_or(0) + 1;
                    let total = self.search.match_count();
                    self.message = Some(format!("[{}/{}]", idx, total));
                } else if self.search.pattern().is_empty() {
                    self.message = Some("No previous search".to_string());
                }
            }
            KeyCode::Char('N') => {
                // Previous match
                if let Some(m) = self.search.jump_prev(&self.cursor) {
                    self.cursor.line = m.line;
                    self.cursor.col = m.start_col;
                    self.ensure_cursor_visible();
                    let idx = self.search.current_match_index().unwrap_or(0) + 1;
                    let total = self.search.match_count();
                    self.message = Some(format!("[{}/{}]", idx, total));
                }
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
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert_char(pos, c);
                self.cursor.col += 1;
            }
            KeyCode::Enter => {
                let pos = self.cursor.byte_offset(&self.buffer);
                self.buffer.insert_char(pos, '\n');
                self.cursor.line += 1;
                self.cursor.col = 0;
                self.ensure_cursor_visible();
            }
            KeyCode::Backspace => {
                let (new_line, new_col) = self.buffer.backspace(self.cursor.line, self.cursor.col);
                self.cursor.line = new_line;
                self.cursor.col = new_col;
                self.ensure_cursor_visible();
            }
            KeyCode::Delete => {
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

    /// Handle keys in search mode
    fn handle_search_mode(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_buffer.clear();
            }
            KeyCode::Enter => {
                // Execute search
                let direction = if self.mode == Mode::SearchForward {
                    SearchDirection::Forward
                } else {
                    SearchDirection::Backward
                };
                
                if !self.search_buffer.is_empty() {
                    self.search.search(&self.search_buffer, direction, &self.buffer);
                    
                    // Jump to first match
                    if let Some(m) = self.search.next_match(&self.cursor, &self.buffer) {
                        self.cursor.line = m.line;
                        self.cursor.col = m.start_col;
                        self.ensure_cursor_visible();
                        
                        let total = self.search.match_count();
                        if total > 0 {
                            let idx = self.search.current_match_index().unwrap_or(0) + 1;
                            self.message = Some(format!("[{}/{}]", idx, total));
                        }
                    } else {
                        self.message = Some(format!("Pattern not found: {}", self.search_buffer));
                    }
                }
                
                self.mode = Mode::Normal;
                self.search_buffer.clear();
            }
            KeyCode::Backspace => {
                if self.search_buffer.pop().is_none() {
                    self.mode = Mode::Normal;
                }
            }
            KeyCode::Char(c) => {
                self.search_buffer.push(c);
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
            _ => {
                self.message = Some(format!("Unknown command: {}", cmd));
            }
        }
        false
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

    pub fn search_buffer(&self) -> &str {
        &self.search_buffer
    }
}
