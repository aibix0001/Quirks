//! Configuration system for Quirks
//!
//! Loads settings from ~/.config/quirks/config.toml

use std::fs;
use std::path::PathBuf;

/// Editor configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Number of spaces per tab
    pub tab_width: usize,
    /// Use spaces instead of tabs
    pub expand_tab: bool,
    /// Show line numbers
    pub line_numbers: bool,
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Show whitespace characters
    pub show_whitespace: bool,
    /// Scroll margin (lines from edge before scrolling)
    pub scroll_margin: usize,
    /// Enable mouse support
    pub mouse: bool,
    /// Color scheme name
    pub colorscheme: String,
    /// Case-insensitive search by default
    pub ignore_case: bool,
    /// Smart case (case-sensitive if pattern has uppercase)
    pub smart_case: bool,
    /// Highlight search matches
    pub hlsearch: bool,
    /// Incremental search
    pub incsearch: bool,
    /// Auto-indent new lines
    pub auto_indent: bool,
    /// Wrap long lines
    pub wrap: bool,
    /// Show cursor line highlight
    pub cursorline: bool,
    /// Clipboard integration (system clipboard)
    pub clipboard: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_width: 4,
            expand_tab: true,
            line_numbers: true,
            syntax_highlighting: true,
            show_whitespace: false,
            scroll_margin: 3,
            mouse: false,
            colorscheme: "default".to_string(),
            ignore_case: false,
            smart_case: true,
            hlsearch: true,
            incsearch: true,
            auto_indent: true,
            wrap: false,
            cursorline: true,
            clipboard: false,
        }
    }
}

impl Config {
    /// Load configuration from file, or return defaults
    pub fn load() -> Self {
        let config_path = Self::config_path();
        
        if let Some(path) = config_path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    return Self::parse(&content);
                }
            }
        }
        
        Self::default()
    }

    /// Get the config file path
    pub fn config_path() -> Option<PathBuf> {
        // Try XDG config first
        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config).join("quirks").join("config.toml");
            return Some(path);
        }
        
        // Fall back to ~/.config
        if let Ok(home) = std::env::var("HOME") {
            let path = PathBuf::from(home)
                .join(".config")
                .join("quirks")
                .join("config.toml");
            return Some(path);
        }
        
        None
    }

    /// Parse configuration from TOML content
    fn parse(content: &str) -> Self {
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            
            // Parse key = value
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "tab_width" => {
                        if let Ok(n) = value.parse() {
                            config.tab_width = n;
                        }
                    }
                    "expand_tab" => {
                        config.expand_tab = value == "true";
                    }
                    "line_numbers" => {
                        config.line_numbers = value == "true";
                    }
                    "syntax_highlighting" => {
                        config.syntax_highlighting = value == "true";
                    }
                    "show_whitespace" => {
                        config.show_whitespace = value == "true";
                    }
                    "scroll_margin" => {
                        if let Ok(n) = value.parse() {
                            config.scroll_margin = n;
                        }
                    }
                    "mouse" => {
                        config.mouse = value == "true";
                    }
                    "colorscheme" => {
                        config.colorscheme = value.trim_matches('"').to_string();
                    }
                    "ignore_case" | "ignorecase" => {
                        config.ignore_case = value == "true";
                    }
                    "smart_case" | "smartcase" => {
                        config.smart_case = value == "true";
                    }
                    "hlsearch" => {
                        config.hlsearch = value == "true";
                    }
                    "incsearch" => {
                        config.incsearch = value == "true";
                    }
                    "auto_indent" | "autoindent" => {
                        config.auto_indent = value == "true";
                    }
                    "wrap" => {
                        config.wrap = value == "true";
                    }
                    "cursorline" => {
                        config.cursorline = value == "true";
                    }
                    "clipboard" => {
                        config.clipboard = value == "true";
                    }
                    _ => {
                        // Unknown key, ignore
                    }
                }
            }
        }
        
        config
    }

    /// Generate default config file content
    pub fn default_config_content() -> &'static str {
        r#"# Quirks Configuration
# Place this file at ~/.config/quirks/config.toml

# Indentation
tab_width = 4
expand_tab = true
auto_indent = true

# Display
line_numbers = true
syntax_highlighting = true
show_whitespace = false
scroll_margin = 3
wrap = false
cursorline = true
colorscheme = "default"

# Search
ignore_case = false
smart_case = true
hlsearch = true
incsearch = true

# Features
mouse = false
clipboard = false
"#
    }

    /// Write default config to file
    pub fn write_default() -> std::io::Result<PathBuf> {
        let path = Self::config_path()
            .ok_or_else(|| std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config path"
            ))?;
        
        // Create parent directories
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&path, Self::default_config_content())?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.tab_width, 4);
        assert!(config.expand_tab);
        assert!(config.line_numbers);
    }

    #[test]
    fn test_parse_config() {
        let content = r#"
            tab_width = 2
            expand_tab = false
            line_numbers = false
            colorscheme = "monokai"
        "#;
        
        let config = Config::parse(content);
        assert_eq!(config.tab_width, 2);
        assert!(!config.expand_tab);
        assert!(!config.line_numbers);
        assert_eq!(config.colorscheme, "monokai");
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
            # This is a comment
            tab_width = 8
            
            # Another comment
            mouse = true
        "#;
        
        let config = Config::parse(content);
        assert_eq!(config.tab_width, 8);
        assert!(config.mouse);
    }
}
