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
    /// Visual Line mode - line-wise selection (V)
    VisualLine,
    /// Visual Block mode - block/column selection (Ctrl+V)
    VisualBlock,
    /// Help mode - shows help overlay
    Help,
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
            Mode::Help => "HELP",
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
            Mode::Help => CursorStyle::Block,
        }
    }

    /// Check if this is a visual mode
    pub fn is_visual(&self) -> bool {
        matches!(self, Mode::Visual | Mode::VisualLine | Mode::VisualBlock)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mode_display() {
        assert_eq!(Mode::Normal.display(), "NORMAL");
        assert_eq!(Mode::Insert.display(), "INSERT");
        assert_eq!(Mode::Command.display(), "COMMAND");
        assert_eq!(Mode::Visual.display(), "VISUAL");
        assert_eq!(Mode::Help.display(), "HELP");
    }

    #[test]
    fn test_mode_is_visual() {
        assert!(!Mode::Normal.is_visual());
        assert!(!Mode::Insert.is_visual());
        assert!(Mode::Visual.is_visual());
        assert!(Mode::VisualLine.is_visual());
        assert!(Mode::VisualBlock.is_visual());
    }

    #[test]
    fn test_mode_default() {
        assert_eq!(Mode::default(), Mode::Normal);
    }
}

/// Cursor visual style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorStyle {
    Block,
    Bar,
    Underline,
}
