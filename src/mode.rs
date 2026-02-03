//! Modal editing modes for Quirks

/// The editing mode determines how keypresses are interpreted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Normal mode - navigation and commands (like Vim)
    #[default]
    Normal,
    /// Insert mode - typing inserts text
    Insert,
    /// Command mode - entering ex-style commands (:w, :q, etc.)
    Command,
    /// Search mode - entering search pattern (/ or ?)
    Search,
    /// Visual mode - character-wise selection (v)
    Visual,
    /// Visual line mode - line-wise selection (V)
    VisualLine,
    /// Visual block mode - block selection (Ctrl+V)
    VisualBlock,
}

impl Mode {
    /// Returns the display name for the status line
    pub fn display(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
            Mode::Search => "SEARCH",
            Mode::Visual => "VISUAL",
            Mode::VisualLine => "V-LINE",
            Mode::VisualBlock => "V-BLOCK",
        }
    }

    /// Returns the cursor style hint for this mode
    pub fn cursor_style(&self) -> CursorStyle {
        match self {
            Mode::Normal => CursorStyle::Block,
            Mode::Insert => CursorStyle::Bar,
            Mode::Command => CursorStyle::Block,
            Mode::Search => CursorStyle::Block,
            Mode::Visual | Mode::VisualLine | Mode::VisualBlock => CursorStyle::Block,
        }
    }

    /// Check if in any visual mode
    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual | Mode::VisualLine | Mode::VisualBlock)
    }
}

/// Cursor visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Bar,
    Underline,
}
