//! Theme and colorscheme system for Quirks
//!
//! Supports built-in themes and loading from files.

use ratatui::style::{Color, Modifier, Style};
use std::collections::HashMap;

/// A color theme for the editor
#[derive(Debug, Clone)]
pub struct Theme {
    /// Theme name
    pub name: String,
    
    // Editor UI colors
    /// Default text
    pub default: Style,
    /// Line numbers
    pub line_number: Style,
    /// Current line number
    pub line_number_current: Style,
    /// Status line
    pub status_line: Style,
    /// Status line (insert mode)
    pub status_line_insert: Style,
    /// Status line (visual mode)
    pub status_line_visual: Style,
    /// Command line
    pub command_line: Style,
    /// Cursor line highlight
    pub cursor_line: Style,
    /// Selection highlight
    pub selection: Style,
    /// Search match highlight
    pub search_match: Style,
    /// Current search match
    pub search_current: Style,
    
    // Syntax highlighting
    /// Keywords (if, else, fn, etc.)
    pub keyword: Style,
    /// Strings
    pub string: Style,
    /// Numbers
    pub number: Style,
    /// Comments
    pub comment: Style,
    /// Function names
    pub function: Style,
    /// Types
    pub type_name: Style,
    /// Operators
    pub operator: Style,
    /// Punctuation
    pub punctuation: Style,
    /// Constants
    pub constant: Style,
    /// Preprocessor/macros
    pub preprocessor: Style,
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

impl Theme {
    /// Default dark theme
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            
            // UI
            default: Style::default().fg(Color::White),
            line_number: Style::default().fg(Color::DarkGray),
            line_number_current: Style::default().fg(Color::Yellow),
            status_line: Style::default().fg(Color::Black).bg(Color::White),
            status_line_insert: Style::default().fg(Color::Black).bg(Color::Green),
            status_line_visual: Style::default().fg(Color::Black).bg(Color::Magenta),
            command_line: Style::default().fg(Color::White),
            cursor_line: Style::default().bg(Color::Rgb(40, 40, 40)),
            selection: Style::default().bg(Color::Rgb(60, 60, 100)),
            search_match: Style::default().bg(Color::Yellow).fg(Color::Black),
            search_current: Style::default().bg(Color::Rgb(255, 150, 0)).fg(Color::Black),
            
            // Syntax
            keyword: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            string: Style::default().fg(Color::Green),
            number: Style::default().fg(Color::Cyan),
            comment: Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
            function: Style::default().fg(Color::Blue),
            type_name: Style::default().fg(Color::Yellow),
            operator: Style::default().fg(Color::White),
            punctuation: Style::default().fg(Color::Gray),
            constant: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            preprocessor: Style::default().fg(Color::Magenta),
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            
            // UI
            default: Style::default().fg(Color::Black),
            line_number: Style::default().fg(Color::Gray),
            line_number_current: Style::default().fg(Color::Blue),
            status_line: Style::default().fg(Color::White).bg(Color::DarkGray),
            status_line_insert: Style::default().fg(Color::White).bg(Color::Green),
            status_line_visual: Style::default().fg(Color::White).bg(Color::Magenta),
            command_line: Style::default().fg(Color::Black),
            cursor_line: Style::default().bg(Color::Rgb(240, 240, 240)),
            selection: Style::default().bg(Color::Rgb(180, 180, 220)),
            search_match: Style::default().bg(Color::Yellow).fg(Color::Black),
            search_current: Style::default().bg(Color::Rgb(255, 180, 0)).fg(Color::Black),
            
            // Syntax
            keyword: Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
            string: Style::default().fg(Color::Rgb(0, 128, 0)),
            number: Style::default().fg(Color::Rgb(0, 128, 128)),
            comment: Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
            function: Style::default().fg(Color::Blue),
            type_name: Style::default().fg(Color::Rgb(128, 64, 0)),
            operator: Style::default().fg(Color::Black),
            punctuation: Style::default().fg(Color::DarkGray),
            constant: Style::default().fg(Color::Rgb(0, 128, 128)).add_modifier(Modifier::BOLD),
            preprocessor: Style::default().fg(Color::Magenta),
        }
    }

    /// Monokai-inspired theme
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            
            // UI
            default: Style::default().fg(Color::Rgb(248, 248, 242)),
            line_number: Style::default().fg(Color::Rgb(117, 113, 94)),
            line_number_current: Style::default().fg(Color::Rgb(248, 248, 242)),
            status_line: Style::default().fg(Color::Rgb(248, 248, 242)).bg(Color::Rgb(64, 64, 64)),
            status_line_insert: Style::default().fg(Color::Black).bg(Color::Rgb(166, 226, 46)),
            status_line_visual: Style::default().fg(Color::Black).bg(Color::Rgb(174, 129, 255)),
            command_line: Style::default().fg(Color::Rgb(248, 248, 242)),
            cursor_line: Style::default().bg(Color::Rgb(60, 60, 50)),
            selection: Style::default().bg(Color::Rgb(73, 72, 62)),
            search_match: Style::default().bg(Color::Rgb(226, 226, 46)).fg(Color::Black),
            search_current: Style::default().bg(Color::Rgb(249, 38, 114)).fg(Color::White),
            
            // Syntax (Monokai colors)
            keyword: Style::default().fg(Color::Rgb(249, 38, 114)),  // Pink
            string: Style::default().fg(Color::Rgb(230, 219, 116)),   // Yellow
            number: Style::default().fg(Color::Rgb(174, 129, 255)),   // Purple
            comment: Style::default().fg(Color::Rgb(117, 113, 94)).add_modifier(Modifier::ITALIC),
            function: Style::default().fg(Color::Rgb(166, 226, 46)),  // Green
            type_name: Style::default().fg(Color::Rgb(102, 217, 239)).add_modifier(Modifier::ITALIC), // Cyan
            operator: Style::default().fg(Color::Rgb(249, 38, 114)),  // Pink
            punctuation: Style::default().fg(Color::Rgb(248, 248, 242)),
            constant: Style::default().fg(Color::Rgb(174, 129, 255)), // Purple
            preprocessor: Style::default().fg(Color::Rgb(249, 38, 114)),
        }
    }

    /// Solarized Dark theme
    pub fn solarized_dark() -> Self {
        // Solarized colors
        let base03 = Color::Rgb(0, 43, 54);
        let base02 = Color::Rgb(7, 54, 66);
        let base01 = Color::Rgb(88, 110, 117);
        let base00 = Color::Rgb(101, 123, 131);
        let base0 = Color::Rgb(131, 148, 150);
        let base1 = Color::Rgb(147, 161, 161);
        let yellow = Color::Rgb(181, 137, 0);
        let orange = Color::Rgb(203, 75, 22);
        let red = Color::Rgb(220, 50, 47);
        let magenta = Color::Rgb(211, 54, 130);
        let violet = Color::Rgb(108, 113, 196);
        let blue = Color::Rgb(38, 139, 210);
        let cyan = Color::Rgb(42, 161, 152);
        let green = Color::Rgb(133, 153, 0);
        
        Self {
            name: "solarized-dark".to_string(),
            
            // UI
            default: Style::default().fg(base0).bg(base03),
            line_number: Style::default().fg(base01),
            line_number_current: Style::default().fg(base1),
            status_line: Style::default().fg(base1).bg(base02),
            status_line_insert: Style::default().fg(base03).bg(green),
            status_line_visual: Style::default().fg(base03).bg(magenta),
            command_line: Style::default().fg(base0),
            cursor_line: Style::default().bg(base02),
            selection: Style::default().bg(base02),
            search_match: Style::default().bg(yellow).fg(base03),
            search_current: Style::default().bg(orange).fg(base03),
            
            // Syntax
            keyword: Style::default().fg(green),
            string: Style::default().fg(cyan),
            number: Style::default().fg(magenta),
            comment: Style::default().fg(base01).add_modifier(Modifier::ITALIC),
            function: Style::default().fg(blue),
            type_name: Style::default().fg(yellow),
            operator: Style::default().fg(green),
            punctuation: Style::default().fg(base0),
            constant: Style::default().fg(violet),
            preprocessor: Style::default().fg(orange),
        }
    }

    /// Nord theme
    pub fn nord() -> Self {
        // Nord colors
        let nord0 = Color::Rgb(46, 52, 64);    // Polar Night
        let nord1 = Color::Rgb(59, 66, 82);
        let nord2 = Color::Rgb(67, 76, 94);
        let nord3 = Color::Rgb(76, 86, 106);
        let nord4 = Color::Rgb(216, 222, 233); // Snow Storm
        let nord5 = Color::Rgb(229, 233, 240);
        let nord6 = Color::Rgb(236, 239, 244);
        let nord7 = Color::Rgb(143, 188, 187); // Frost
        let nord8 = Color::Rgb(136, 192, 208);
        let nord9 = Color::Rgb(129, 161, 193);
        let nord10 = Color::Rgb(94, 129, 172);
        let nord11 = Color::Rgb(191, 97, 106); // Aurora
        let nord12 = Color::Rgb(208, 135, 112);
        let nord13 = Color::Rgb(235, 203, 139);
        let nord14 = Color::Rgb(163, 190, 140);
        let nord15 = Color::Rgb(180, 142, 173);
        
        Self {
            name: "nord".to_string(),
            
            // UI
            default: Style::default().fg(nord4).bg(nord0),
            line_number: Style::default().fg(nord3),
            line_number_current: Style::default().fg(nord4),
            status_line: Style::default().fg(nord4).bg(nord1),
            status_line_insert: Style::default().fg(nord0).bg(nord14),
            status_line_visual: Style::default().fg(nord0).bg(nord15),
            command_line: Style::default().fg(nord4),
            cursor_line: Style::default().bg(nord1),
            selection: Style::default().bg(nord2),
            search_match: Style::default().bg(nord13).fg(nord0),
            search_current: Style::default().bg(nord12).fg(nord0),
            
            // Syntax
            keyword: Style::default().fg(nord9),
            string: Style::default().fg(nord14),
            number: Style::default().fg(nord15),
            comment: Style::default().fg(nord3).add_modifier(Modifier::ITALIC),
            function: Style::default().fg(nord8),
            type_name: Style::default().fg(nord7),
            operator: Style::default().fg(nord9),
            punctuation: Style::default().fg(nord4),
            constant: Style::default().fg(nord15),
            preprocessor: Style::default().fg(nord10),
        }
    }

    /// Get a theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "dark" | "default" => Some(Self::dark()),
            "light" => Some(Self::light()),
            "monokai" => Some(Self::monokai()),
            "solarized" | "solarized-dark" => Some(Self::solarized_dark()),
            "nord" => Some(Self::nord()),
            _ => None,
        }
    }

    /// List available theme names
    pub fn available() -> Vec<&'static str> {
        vec!["dark", "light", "monokai", "solarized-dark", "nord"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme() {
        let theme = Theme::default();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_by_name() {
        assert!(Theme::by_name("dark").is_some());
        assert!(Theme::by_name("light").is_some());
        assert!(Theme::by_name("monokai").is_some());
        assert!(Theme::by_name("nord").is_some());
        assert!(Theme::by_name("nonexistent").is_none());
    }

    #[test]
    fn test_available_themes() {
        let themes = Theme::available();
        assert!(themes.contains(&"dark"));
        assert!(themes.contains(&"light"));
        assert!(themes.len() >= 4);
    }
}
