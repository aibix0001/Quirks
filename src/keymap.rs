//! Custom keybinding system for Quirks
//!
//! Allows users to remap keys and define custom commands.

use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;

/// A key combination (key + modifiers)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyCombo {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    pub fn plain(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::NONE)
    }

    pub fn ctrl(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::CONTROL)
    }

    pub fn shift(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::SHIFT)
    }

    pub fn alt(code: KeyCode) -> Self {
        Self::new(code, KeyModifiers::ALT)
    }

    /// Parse a key string like "Ctrl+s", "j", "Shift+Tab"
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim();
        let parts: Vec<&str> = s.split('+').collect();
        
        let mut modifiers = KeyModifiers::NONE;
        let mut key_part = "";
        
        for part in &parts {
            let part = part.trim();
            match part.to_lowercase().as_str() {
                "ctrl" | "control" if parts.len() > 1 => {
                    modifiers |= KeyModifiers::CONTROL;
                }
                "shift" if parts.len() > 1 => {
                    modifiers |= KeyModifiers::SHIFT;
                }
                "alt" | "meta" if parts.len() > 1 => {
                    modifiers |= KeyModifiers::ALT;
                }
                _ => {
                    key_part = part;
                }
            }
        }
        
        let code = parse_key_code(key_part)?;
        Some(Self::new(code, modifiers))
    }
}

/// Parse a key code from string
fn parse_key_code(s: &str) -> Option<KeyCode> {
    let s = s.to_lowercase();
    match s.as_str() {
        // Special keys
        "esc" | "escape" => Some(KeyCode::Esc),
        "enter" | "return" | "cr" => Some(KeyCode::Enter),
        "tab" => Some(KeyCode::Tab),
        "backspace" | "bs" => Some(KeyCode::Backspace),
        "delete" | "del" => Some(KeyCode::Delete),
        "insert" | "ins" => Some(KeyCode::Insert),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "pageup" | "pgup" => Some(KeyCode::PageUp),
        "pagedown" | "pgdn" => Some(KeyCode::PageDown),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "space" => Some(KeyCode::Char(' ')),
        
        // Function keys
        "f1" => Some(KeyCode::F(1)),
        "f2" => Some(KeyCode::F(2)),
        "f3" => Some(KeyCode::F(3)),
        "f4" => Some(KeyCode::F(4)),
        "f5" => Some(KeyCode::F(5)),
        "f6" => Some(KeyCode::F(6)),
        "f7" => Some(KeyCode::F(7)),
        "f8" => Some(KeyCode::F(8)),
        "f9" => Some(KeyCode::F(9)),
        "f10" => Some(KeyCode::F(10)),
        "f11" => Some(KeyCode::F(11)),
        "f12" => Some(KeyCode::F(12)),
        
        // Single characters
        _ if s.len() == 1 => {
            s.chars().next().map(KeyCode::Char)
        }
        
        _ => None,
    }
}

/// Actions that can be bound to keys
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    // Movement
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    MoveWordForward,
    MoveWordBackward,
    MoveLineStart,
    MoveLineEnd,
    MoveBufferStart,
    MoveBufferEnd,
    PageUp,
    PageDown,
    
    // Mode switching
    EnterInsert,
    EnterInsertAfter,
    EnterInsertLineStart,
    EnterInsertLineEnd,
    EnterNormal,
    EnterCommand,
    EnterVisual,
    EnterVisualLine,
    
    // Editing
    DeleteChar,
    DeleteLine,
    YankLine,
    Paste,
    PasteBefore,
    Undo,
    Redo,
    NewLineBelow,
    NewLineAbove,
    JoinLines,
    
    // Search
    SearchForward,
    SearchBackward,
    NextMatch,
    PrevMatch,
    ClearSearch,
    
    // File operations
    Save,
    SaveAs,
    Quit,
    ForceQuit,
    SaveAndQuit,
    
    // Custom command
    Command(String),
    
    // No operation
    Noop,
}

/// Keymap for a specific mode
#[derive(Debug, Clone)]
pub struct ModeKeymap {
    bindings: HashMap<KeyCombo, Action>,
}

impl ModeKeymap {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }

    pub fn bind(&mut self, key: KeyCombo, action: Action) {
        self.bindings.insert(key, action);
    }

    pub fn unbind(&mut self, key: &KeyCombo) {
        self.bindings.remove(key);
    }

    pub fn get(&self, key: &KeyCombo) -> Option<&Action> {
        self.bindings.get(key)
    }

    pub fn with_defaults_normal() -> Self {
        let mut km = Self::new();
        
        // Movement
        km.bind(KeyCombo::plain(KeyCode::Char('h')), Action::MoveLeft);
        km.bind(KeyCombo::plain(KeyCode::Char('j')), Action::MoveDown);
        km.bind(KeyCombo::plain(KeyCode::Char('k')), Action::MoveUp);
        km.bind(KeyCombo::plain(KeyCode::Char('l')), Action::MoveRight);
        km.bind(KeyCombo::plain(KeyCode::Left), Action::MoveLeft);
        km.bind(KeyCombo::plain(KeyCode::Down), Action::MoveDown);
        km.bind(KeyCombo::plain(KeyCode::Up), Action::MoveUp);
        km.bind(KeyCombo::plain(KeyCode::Right), Action::MoveRight);
        km.bind(KeyCombo::plain(KeyCode::Char('w')), Action::MoveWordForward);
        km.bind(KeyCombo::plain(KeyCode::Char('b')), Action::MoveWordBackward);
        km.bind(KeyCombo::plain(KeyCode::Char('0')), Action::MoveLineStart);
        km.bind(KeyCombo::plain(KeyCode::Char('$')), Action::MoveLineEnd);
        km.bind(KeyCombo::plain(KeyCode::Char('g')), Action::MoveBufferStart);
        km.bind(KeyCombo::shift(KeyCode::Char('G')), Action::MoveBufferEnd);
        km.bind(KeyCombo::ctrl(KeyCode::Char('u')), Action::PageUp);
        km.bind(KeyCombo::ctrl(KeyCode::Char('d')), Action::PageDown);
        
        // Mode switching
        km.bind(KeyCombo::plain(KeyCode::Char('i')), Action::EnterInsert);
        km.bind(KeyCombo::plain(KeyCode::Char('a')), Action::EnterInsertAfter);
        km.bind(KeyCombo::shift(KeyCode::Char('I')), Action::EnterInsertLineStart);
        km.bind(KeyCombo::shift(KeyCode::Char('A')), Action::EnterInsertLineEnd);
        km.bind(KeyCombo::plain(KeyCode::Char(':')), Action::EnterCommand);
        km.bind(KeyCombo::plain(KeyCode::Char('v')), Action::EnterVisual);
        km.bind(KeyCombo::shift(KeyCode::Char('V')), Action::EnterVisualLine);
        
        // Editing
        km.bind(KeyCombo::plain(KeyCode::Char('x')), Action::DeleteChar);
        km.bind(KeyCombo::plain(KeyCode::Char('p')), Action::Paste);
        km.bind(KeyCombo::shift(KeyCode::Char('P')), Action::PasteBefore);
        km.bind(KeyCombo::plain(KeyCode::Char('u')), Action::Undo);
        km.bind(KeyCombo::ctrl(KeyCode::Char('r')), Action::Redo);
        km.bind(KeyCombo::plain(KeyCode::Char('o')), Action::NewLineBelow);
        km.bind(KeyCombo::shift(KeyCode::Char('O')), Action::NewLineAbove);
        km.bind(KeyCombo::shift(KeyCode::Char('J')), Action::JoinLines);
        
        // Search
        km.bind(KeyCombo::plain(KeyCode::Char('/')), Action::SearchForward);
        km.bind(KeyCombo::plain(KeyCode::Char('?')), Action::SearchBackward);
        km.bind(KeyCombo::plain(KeyCode::Char('n')), Action::NextMatch);
        km.bind(KeyCombo::shift(KeyCode::Char('N')), Action::PrevMatch);
        
        km
    }

    pub fn with_defaults_insert() -> Self {
        let mut km = Self::new();
        
        km.bind(KeyCombo::plain(KeyCode::Esc), Action::EnterNormal);
        km.bind(KeyCombo::ctrl(KeyCode::Char('c')), Action::EnterNormal);
        
        km
    }
}

impl Default for ModeKeymap {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete keymap with all modes
#[derive(Debug, Clone)]
pub struct Keymap {
    pub normal: ModeKeymap,
    pub insert: ModeKeymap,
    pub visual: ModeKeymap,
    pub command: ModeKeymap,
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            normal: ModeKeymap::with_defaults_normal(),
            insert: ModeKeymap::with_defaults_insert(),
            visual: ModeKeymap::with_defaults_normal(), // Visual uses similar bindings
            command: ModeKeymap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_combo_parse() {
        let combo = KeyCombo::parse("Ctrl+s").unwrap();
        assert_eq!(combo.code, KeyCode::Char('s'));
        assert!(combo.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_key_combo_parse_plain() {
        let combo = KeyCombo::parse("j").unwrap();
        assert_eq!(combo.code, KeyCode::Char('j'));
        assert_eq!(combo.modifiers, KeyModifiers::NONE);
    }

    #[test]
    fn test_key_combo_parse_special() {
        let combo = KeyCombo::parse("Escape").unwrap();
        assert_eq!(combo.code, KeyCode::Esc);
    }

    #[test]
    fn test_key_combo_parse_function() {
        let combo = KeyCombo::parse("F5").unwrap();
        assert_eq!(combo.code, KeyCode::F(5));
    }

    #[test]
    fn test_mode_keymap_defaults() {
        let km = ModeKeymap::with_defaults_normal();
        
        let action = km.get(&KeyCombo::plain(KeyCode::Char('j')));
        assert_eq!(action, Some(&Action::MoveDown));
    }

    #[test]
    fn test_mode_keymap_custom() {
        let mut km = ModeKeymap::new();
        km.bind(KeyCombo::ctrl(KeyCode::Char('s')), Action::Save);
        
        let action = km.get(&KeyCombo::ctrl(KeyCode::Char('s')));
        assert_eq!(action, Some(&Action::Save));
    }
}

/// Convert a Lua-style keymap string to a KeyCombo
/// Supports: "<leader>w", "<C-s>", "<M-x>", "<S-Tab>", "j", etc.
pub fn parse_vim_key(s: &str) -> Option<KeyCombo> {
    let s = s.trim();
    
    // Handle <...> notation
    if s.starts_with('<') && s.ends_with('>') {
        let inner = &s[1..s.len()-1];
        
        // Handle modifier combinations like <C-s>, <M-x>, <S-Tab>
        if inner.contains('-') {
            let parts: Vec<&str> = inner.splitn(2, '-').collect();
            if parts.len() == 2 {
                let modifier_str = parts[0].to_uppercase();
                let key_str = parts[1];
                
                let mut modifiers = KeyModifiers::NONE;
                for c in modifier_str.chars() {
                    match c {
                        'C' => modifiers |= KeyModifiers::CONTROL,
                        'S' => modifiers |= KeyModifiers::SHIFT,
                        'M' | 'A' => modifiers |= KeyModifiers::ALT,
                        _ => {}
                    }
                }
                
                let code = parse_key_code(key_str)?;
                return Some(KeyCombo::new(code, modifiers));
            }
        }
        
        // Handle special keys like <CR>, <Esc>, <Tab>
        match inner.to_lowercase().as_str() {
            "cr" | "enter" | "return" => return Some(KeyCombo::plain(KeyCode::Enter)),
            "esc" | "escape" => return Some(KeyCombo::plain(KeyCode::Esc)),
            "tab" => return Some(KeyCombo::plain(KeyCode::Tab)),
            "space" => return Some(KeyCombo::plain(KeyCode::Char(' '))),
            "bs" | "backspace" => return Some(KeyCombo::plain(KeyCode::Backspace)),
            "leader" => return Some(KeyCombo::plain(KeyCode::Char(' '))), // Space as leader
            _ => {}
        }
    }
    
    // Plain key
    if s.len() == 1 {
        return Some(KeyCombo::plain(KeyCode::Char(s.chars().next()?)));
    }
    
    // Fallback to existing parser
    KeyCombo::parse(s)
}

#[cfg(test)]
mod vim_key_tests {
    use super::*;

    #[test]
    fn test_parse_vim_ctrl() {
        let combo = parse_vim_key("<C-s>").unwrap();
        assert_eq!(combo.code, KeyCode::Char('s'));
        assert!(combo.modifiers.contains(KeyModifiers::CONTROL));
    }

    #[test]
    fn test_parse_vim_cr() {
        let combo = parse_vim_key("<CR>").unwrap();
        assert_eq!(combo.code, KeyCode::Enter);
    }

    #[test]
    fn test_parse_vim_plain() {
        let combo = parse_vim_key("j").unwrap();
        assert_eq!(combo.code, KeyCode::Char('j'));
    }
}
