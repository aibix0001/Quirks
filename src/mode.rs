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
    /// Search forward mode (/)
    SearchForward,
    /// Search backward mode (?)
    SearchBackward,
}

impl Mode {
    /// Returns the display name for the status line
    pub fn display(&self) -> &'static str {
        match self {
            Mode::Normal => "NORMAL",
            Mode::Insert => "INSERT",
            Mode::Command => "COMMAND",
            Mode::SearchForward => "SEARCH",
            Mode::SearchBackward => "SEARCH",
        }
    }

    /// Returns the cursor style hint for this mode
    pub fn cursor_style(&self) -> CursorStyle {
        match self {
            Mode::Normal => CursorStyle::Block,
            Mode::Insert => CursorStyle::Bar,
            Mode::Command => CursorStyle::Block,
            Mode::SearchForward | Mode::SearchBackward => CursorStyle::Block,
        }
    }
    
    /// Check if in any search mode
    pub fn is_search(&self) -> bool {
        matches!(self, Mode::SearchForward | Mode::SearchBackward)
    }
}

/// Cursor visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Bar,
    Underline,
}
