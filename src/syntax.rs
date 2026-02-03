//! Syntax highlighting for Quirks
//!
//! Simple regex-based syntax highlighting. Tree-sitter integration planned for v0.2.

use ratatui::style::{Color, Style};
use std::collections::HashMap;

/// A syntax highlighting rule
#[derive(Debug, Clone)]
pub struct SyntaxRule {
    /// Regex pattern to match
    pub pattern: &'static str,
    /// Style to apply
    pub style: Style,
}

/// Syntax definition for a language
#[derive(Debug, Clone)]
pub struct SyntaxDef {
    /// File extensions this syntax applies to
    pub extensions: &'static [&'static str],
    /// Name of the language
    pub name: &'static str,
    /// Keywords
    pub keywords: &'static [&'static str],
    /// Types
    pub types: &'static [&'static str],
    /// Single-line comment prefix
    pub comment_single: Option<&'static str>,
    /// Multi-line comment delimiters (start, end)
    pub comment_multi: Option<(&'static str, &'static str)>,
    /// String delimiters
    pub string_delimiters: &'static [char],
}

/// Highlighted span within a line
#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start: usize,
    pub end: usize,
    pub style: Style,
}

/// The syntax highlighter
pub struct Highlighter {
    /// Available syntax definitions
    syntaxes: HashMap<&'static str, SyntaxDef>,
    /// Current active syntax (by extension)
    current: Option<&'static str>,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl Highlighter {
    pub fn new() -> Self {
        let mut syntaxes = HashMap::new();
        
        // Rust syntax
        syntaxes.insert("rs", SyntaxDef {
            extensions: &["rs"],
            name: "Rust",
            keywords: &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn",
                "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in",
                "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
                "self", "Self", "static", "struct", "super", "trait", "true", "type",
                "unsafe", "use", "where", "while",
            ],
            types: &[
                "bool", "char", "str", "u8", "u16", "u32", "u64", "u128", "usize",
                "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64",
                "String", "Vec", "Option", "Result", "Box", "Rc", "Arc",
            ],
            comment_single: Some("//"),
            comment_multi: Some(("/*", "*/")),
            string_delimiters: &['"'],
        });

        // Python syntax
        syntaxes.insert("py", SyntaxDef {
            extensions: &["py", "pyw"],
            name: "Python",
            keywords: &[
                "and", "as", "assert", "async", "await", "break", "class", "continue",
                "def", "del", "elif", "else", "except", "finally", "for", "from",
                "global", "if", "import", "in", "is", "lambda", "nonlocal", "not",
                "or", "pass", "raise", "return", "try", "while", "with", "yield",
                "True", "False", "None",
            ],
            types: &["int", "float", "str", "bool", "list", "dict", "tuple", "set"],
            comment_single: Some("#"),
            comment_multi: None,
            string_delimiters: &['"', '\''],
        });

        // JavaScript/TypeScript
        syntaxes.insert("js", SyntaxDef {
            extensions: &["js", "jsx", "ts", "tsx"],
            name: "JavaScript",
            keywords: &[
                "break", "case", "catch", "class", "const", "continue", "debugger",
                "default", "delete", "do", "else", "export", "extends", "finally",
                "for", "function", "if", "import", "in", "instanceof", "let", "new",
                "return", "super", "switch", "this", "throw", "try", "typeof", "var",
                "void", "while", "with", "yield", "async", "await", "of",
                "true", "false", "null", "undefined",
            ],
            types: &["string", "number", "boolean", "object", "Array", "Promise"],
            comment_single: Some("//"),
            comment_multi: Some(("/*", "*/")),
            string_delimiters: &['"', '\'', '`'],
        });

        // Markdown
        syntaxes.insert("md", SyntaxDef {
            extensions: &["md", "markdown"],
            name: "Markdown",
            keywords: &[],
            types: &[],
            comment_single: None,
            comment_multi: None,
            string_delimiters: &[],
        });

        // TOML
        syntaxes.insert("toml", SyntaxDef {
            extensions: &["toml"],
            name: "TOML",
            keywords: &["true", "false"],
            types: &[],
            comment_single: Some("#"),
            comment_multi: None,
            string_delimiters: &['"', '\''],
        });

        Self {
            syntaxes,
            current: None,
        }
    }

    /// Set the current syntax based on file extension
    pub fn set_syntax_for_extension(&mut self, ext: &str) {
        let ext = ext.trim_start_matches('.');
        self.current = self.syntaxes.keys().find(|&&k| {
            self.syntaxes.get(k).map_or(false, |s| s.extensions.contains(&ext))
        }).copied();
    }

    /// Get the current syntax name
    pub fn current_syntax_name(&self) -> Option<&'static str> {
        self.current.and_then(|ext| self.syntaxes.get(ext).map(|s| s.name))
    }

    /// Highlight a line of text
    pub fn highlight_line(&self, line: &str) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();
        
        let syntax = match self.current.and_then(|ext| self.syntaxes.get(ext)) {
            Some(s) => s,
            None => return spans, // No highlighting
        };

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        // Helper: check if chars starting at index match a prefix string
        let chars_start_with = |chars: &[char], idx: usize, prefix: &str| -> bool {
            let prefix_chars: Vec<char> = prefix.chars().collect();
            if idx + prefix_chars.len() > chars.len() {
                return false;
            }
            chars[idx..idx + prefix_chars.len()] == prefix_chars[..]
        };

        while i < chars.len() {
            // Check for comments
            if let Some(comment_prefix) = syntax.comment_single {
                if chars_start_with(&chars, i, comment_prefix) {
                    spans.push(HighlightSpan {
                        start: i,
                        end: chars.len(),
                        style: Style::default().fg(Color::DarkGray),
                    });
                    break;
                }
            }

            // Check for strings
            if syntax.string_delimiters.contains(&chars[i]) {
                let delim = chars[i];
                let start = i;
                i += 1;
                while i < chars.len() && chars[i] != delim {
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        i += 1; // Skip escaped char
                    }
                    i += 1;
                }
                if i < chars.len() {
                    i += 1; // Include closing delimiter
                }
                spans.push(HighlightSpan {
                    start,
                    end: i,
                    style: Style::default().fg(Color::Green),
                });
                continue;
            }

            // Check for numbers
            if chars[i].is_ascii_digit() {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '.' || chars[i] == '_') {
                    i += 1;
                }
                spans.push(HighlightSpan {
                    start,
                    end: i,
                    style: Style::default().fg(Color::Magenta),
                });
                continue;
            }

            // Check for identifiers (keywords, types)
            if chars[i].is_alphabetic() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                
                if syntax.keywords.contains(&word.as_str()) {
                    spans.push(HighlightSpan {
                        start,
                        end: i,
                        style: Style::default().fg(Color::Yellow),
                    });
                } else if syntax.types.contains(&word.as_str()) {
                    spans.push(HighlightSpan {
                        start,
                        end: i,
                        style: Style::default().fg(Color::Cyan),
                    });
                }
                continue;
            }

            i += 1;
        }

        spans
    }
}

// Color scheme constants
pub mod colors {
    use ratatui::style::Color;

    pub const KEYWORD: Color = Color::Yellow;
    pub const TYPE: Color = Color::Cyan;
    pub const STRING: Color = Color::Green;
    pub const NUMBER: Color = Color::Magenta;
    pub const COMMENT: Color = Color::DarkGray;
    pub const FUNCTION: Color = Color::Blue;
    pub const OPERATOR: Color = Color::White;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_highlighter_new() {
        let highlighter = Highlighter::new();
        assert!(highlighter.current_syntax_name().is_none());
    }

    #[test]
    fn test_highlighter_set_syntax() {
        let mut highlighter = Highlighter::new();
        highlighter.set_syntax_for_extension("rs");
        assert_eq!(highlighter.current_syntax_name(), Some("Rust"));
    }

    #[test]
    fn test_highlighter_python() {
        let mut highlighter = Highlighter::new();
        highlighter.set_syntax_for_extension("py");
        assert_eq!(highlighter.current_syntax_name(), Some("Python"));
    }

    #[test]
    fn test_highlighter_unknown() {
        let mut highlighter = Highlighter::new();
        highlighter.set_syntax_for_extension("xyz");
        assert!(highlighter.current_syntax_name().is_none());
    }

    #[test]
    fn test_highlight_rust_keyword() {
        let mut highlighter = Highlighter::new();
        highlighter.set_syntax_for_extension("rs");
        let spans = highlighter.highlight_line("fn main() {");
        assert!(!spans.is_empty());
    }

    #[test]
    fn test_highlight_comment() {
        let mut highlighter = Highlighter::new();
        highlighter.set_syntax_for_extension("rs");
        let spans = highlighter.highlight_line("// this is a comment");
        assert!(!spans.is_empty());
    }
}
