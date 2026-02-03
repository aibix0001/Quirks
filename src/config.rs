//! Configuration system for Quirks
//!
//! Loads settings from ~/.quirksrc or ~/.config/quirks/config.toml

use std::fs;
use std::path::PathBuf;

/// Editor configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Tab width in spaces
    pub tab_width: usize,
    /// Show line numbers
    pub line_numbers: bool,
    /// Enable syntax highlighting
    pub syntax_highlighting: bool,
    /// Auto-indent new lines
    pub auto_indent: bool,
    /// Show whitespace characters
    pub show_whitespace: bool,
    /// Color scheme name
    pub color_scheme: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tab_width: 4,
            line_numbers: true,
            syntax_highlighting: true,
            auto_indent: true,
            show_whitespace: false,
            color_scheme: "default".to_string(),
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> Self {
        // Try ~/.quirksrc first
        if let Some(home) = dirs::home_dir() {
            let quirksrc = home.join(".quirksrc");
            if quirksrc.exists() {
                if let Ok(config) = Self::from_file(&quirksrc) {
                    return config;
                }
            }

            // Try ~/.config/quirks/config.toml
            let config_dir = home.join(".config").join("quirks").join("config.toml");
            if config_dir.exists() {
                if let Ok(config) = Self::from_file(&config_dir) {
                    return config;
                }
            }
        }

        Self::default()
    }

    /// Parse configuration from file
    fn from_file(path: &PathBuf) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let mut config = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim().trim_matches('"');

                match key {
                    "tab_width" => {
                        if let Ok(n) = value.parse() {
                            config.tab_width = n;
                        }
                    }
                    "line_numbers" => {
                        config.line_numbers = value == "true" || value == "1";
                    }
                    "syntax_highlighting" => {
                        config.syntax_highlighting = value == "true" || value == "1";
                    }
                    "auto_indent" => {
                        config.auto_indent = value == "true" || value == "1";
                    }
                    "show_whitespace" => {
                        config.show_whitespace = value == "true" || value == "1";
                    }
                    "color_scheme" => {
                        config.color_scheme = value.to_string();
                    }
                    _ => {} // Ignore unknown keys
                }
            }
        }

        Ok(config)
    }

    /// Get config file path for user reference
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".quirksrc"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.tab_width, 4);
        assert!(config.line_numbers);
        assert!(config.syntax_highlighting);
    }
}
