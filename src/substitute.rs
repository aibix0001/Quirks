//! Search and replace functionality for Quirks
//!
//! Implements Vim-style :%s/pattern/replacement/flags

use regex::Regex;

/// Result of a substitution operation
#[derive(Debug)]
pub struct SubstituteResult {
    /// Number of replacements made
    pub count: usize,
    /// Number of lines affected
    pub lines: usize,
    /// Error message if any
    pub error: Option<String>,
}

/// Flags for substitution
#[derive(Debug, Default)]
pub struct SubstituteFlags {
    /// Replace all occurrences on each line (g flag)
    pub global: bool,
    /// Case insensitive matching (i flag)
    pub ignore_case: bool,
    /// Confirm each replacement (c flag) - not yet implemented
    pub confirm: bool,
    /// Print matching lines (p flag) - not yet implemented  
    pub print: bool,
}

impl SubstituteFlags {
    /// Parse flags from string (e.g., "gi")
    pub fn parse(s: &str) -> Self {
        let mut flags = Self::default();
        for ch in s.chars() {
            match ch {
                'g' => flags.global = true,
                'i' | 'I' => flags.ignore_case = true,
                'c' => flags.confirm = true,
                'p' => flags.print = true,
                _ => {}
            }
        }
        flags
    }
}

/// Parse a substitute command
/// Format: s/pattern/replacement/flags or %s/pattern/replacement/flags
pub fn parse_substitute_command(cmd: &str) -> Option<(Option<Range>, String, String, SubstituteFlags)> {
    let cmd = cmd.trim();
    
    // Check for range prefix
    let (range, rest) = if cmd.starts_with('%') {
        (Some(Range::All), &cmd[1..])
    } else if cmd.starts_with('.') {
        (Some(Range::Current), &cmd[1..])
    } else if let Some(stripped) = cmd.strip_prefix("s") {
        (Some(Range::Current), cmd)
    } else {
        (None, cmd)
    };
    
    // Must start with 's'
    let rest = rest.strip_prefix('s')?;
    
    // Get delimiter (usually /)
    let delim = rest.chars().next()?;
    let rest = &rest[delim.len_utf8()..];
    
    // Split by delimiter
    let parts: Vec<&str> = split_by_delimiter(rest, delim);
    
    if parts.len() < 2 {
        return None;
    }
    
    let pattern = parts[0].to_string();
    let replacement = parts[1].to_string();
    let flags = if parts.len() > 2 {
        SubstituteFlags::parse(parts[2])
    } else {
        SubstituteFlags::default()
    };
    
    Some((range, pattern, replacement, flags))
}

/// Split string by delimiter, respecting escapes
fn split_by_delimiter(s: &str, delim: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut escaped = false;
    
    for (i, ch) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == delim {
            parts.push(&s[start..i]);
            start = i + ch.len_utf8();
        }
    }
    
    // Add the remaining part
    if start <= s.len() {
        parts.push(&s[start..]);
    }
    
    parts
}

/// Range for substitution
#[derive(Debug, Clone, Copy)]
pub enum Range {
    /// Current line only
    Current,
    /// All lines (%)
    All,
    /// Specific line number
    Line(usize),
    /// Range of lines (start, end)
    Lines(usize, usize),
}

/// Perform substitution on text
pub fn substitute(
    lines: &mut Vec<String>,
    range: Range,
    pattern: &str,
    replacement: &str,
    flags: &SubstituteFlags,
    current_line: usize,
) -> SubstituteResult {
    // Build regex
    let regex = match if flags.ignore_case {
        Regex::new(&format!("(?i){}", pattern))
    } else {
        Regex::new(pattern)
    } {
        Ok(r) => r,
        Err(e) => {
            return SubstituteResult {
                count: 0,
                lines: 0,
                error: Some(format!("Invalid pattern: {}", e)),
            };
        }
    };
    
    // Determine line range
    let (start, end) = match range {
        Range::Current => (current_line, current_line),
        Range::All => (0, lines.len().saturating_sub(1)),
        Range::Line(n) => (n, n),
        Range::Lines(s, e) => (s, e),
    };
    
    let mut total_count = 0;
    let mut affected_lines = 0;
    
    for line_idx in start..=end.min(lines.len().saturating_sub(1)) {
        let line = &lines[line_idx];
        let new_line = if flags.global {
            regex.replace_all(line, replacement).to_string()
        } else {
            regex.replace(line, replacement).to_string()
        };
        
        if new_line != *line {
            let count = if flags.global {
                regex.find_iter(line).count()
            } else {
                1
            };
            total_count += count;
            affected_lines += 1;
            lines[line_idx] = new_line;
        }
    }
    
    SubstituteResult {
        count: total_count,
        lines: affected_lines,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_substitute_basic() {
        let result = parse_substitute_command("s/foo/bar/");
        assert!(result.is_some());
        let (range, pattern, replacement, flags) = result.unwrap();
        assert_eq!(pattern, "foo");
        assert_eq!(replacement, "bar");
        assert!(!flags.global);
    }

    #[test]
    fn test_parse_substitute_global() {
        let result = parse_substitute_command("%s/foo/bar/g");
        assert!(result.is_some());
        let (range, pattern, replacement, flags) = result.unwrap();
        assert!(matches!(range, Some(Range::All)));
        assert_eq!(pattern, "foo");
        assert_eq!(replacement, "bar");
        assert!(flags.global);
    }

    #[test]
    fn test_parse_substitute_flags() {
        let result = parse_substitute_command("s/foo/bar/gi");
        assert!(result.is_some());
        let (_, _, _, flags) = result.unwrap();
        assert!(flags.global);
        assert!(flags.ignore_case);
    }

    #[test]
    fn test_substitute_single() {
        let mut lines = vec![
            "hello world".to_string(),
            "hello again".to_string(),
        ];
        let flags = SubstituteFlags::default();
        
        let result = substitute(&mut lines, Range::Current, "hello", "hi", &flags, 0);
        
        assert_eq!(result.count, 1);
        assert_eq!(lines[0], "hi world");
        assert_eq!(lines[1], "hello again"); // Unchanged
    }

    #[test]
    fn test_substitute_all_lines() {
        let mut lines = vec![
            "hello world".to_string(),
            "hello again".to_string(),
        ];
        let flags = SubstituteFlags::default();
        
        let result = substitute(&mut lines, Range::All, "hello", "hi", &flags, 0);
        
        assert_eq!(result.count, 2);
        assert_eq!(result.lines, 2);
        assert_eq!(lines[0], "hi world");
        assert_eq!(lines[1], "hi again");
    }

    #[test]
    fn test_substitute_global_flag() {
        let mut lines = vec!["aaa".to_string()];
        let mut flags = SubstituteFlags::default();
        flags.global = true;
        
        let result = substitute(&mut lines, Range::Current, "a", "b", &flags, 0);
        
        assert_eq!(lines[0], "bbb");
    }

    #[test]
    fn test_substitute_regex() {
        let mut lines = vec!["hello123world".to_string()];
        let flags = SubstituteFlags::default();
        
        let result = substitute(&mut lines, Range::Current, r"\d+", "###", &flags, 0);
        
        assert_eq!(lines[0], "hello###world");
    }

    #[test]
    fn test_split_by_delimiter() {
        let parts = split_by_delimiter("foo/bar/baz", '/');
        assert_eq!(parts, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_split_with_escaped() {
        let parts = split_by_delimiter(r"foo\/bar/baz", '/');
        assert_eq!(parts, vec![r"foo\/bar", "baz"]);
    }
}
