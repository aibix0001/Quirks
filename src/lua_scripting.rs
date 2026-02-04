//! Lua scripting integration for Quirks
//!
//! Provides Lua 5.4 scripting support for extending the editor.
//! Users can create custom commands, keybindings, and automation scripts.
//!
//! # Example Lua Script
//! ```lua
//! -- ~/.config/quirks/init.lua
//! quirks.set_option("number", true)
//! quirks.set_option("relativenumber", true)
//!
//! quirks.keymap("n", "<leader>w", ":w<CR>")
//! quirks.command("hello", function()
//!     quirks.print("Hello from Lua!")
//! end)
//! ```

use mlua::{Lua, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Result type for Lua operations
pub type LuaResult<T> = Result<T, LuaError>;

/// Error type for Lua operations
#[derive(Debug)]
pub enum LuaError {
    /// Lua runtime error
    Runtime(String),
    /// Configuration file error
    Config(String),
    /// IO error
    Io(std::io::Error),
}

impl std::fmt::Display for LuaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LuaError::Runtime(msg) => write!(f, "Lua error: {}", msg),
            LuaError::Config(msg) => write!(f, "Config error: {}", msg),
            LuaError::Io(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl std::error::Error for LuaError {}

impl From<mlua::Error> for LuaError {
    fn from(err: mlua::Error) -> Self {
        LuaError::Runtime(err.to_string())
    }
}

impl From<std::io::Error> for LuaError {
    fn from(err: std::io::Error) -> Self {
        LuaError::Io(err)
    }
}

/// Lua scripting engine for Quirks
pub struct LuaEngine {
    lua: Lua,
    /// Custom commands registered via Lua
    custom_commands: Arc<Mutex<HashMap<String, String>>>,
    /// Custom keymaps registered via Lua
    custom_keymaps: Arc<Mutex<Vec<Keymap>>>,
    /// Messages to display to user
    messages: Arc<Mutex<Vec<String>>>,
}

/// A custom keymap definition
#[derive(Debug, Clone)]
pub struct Keymap {
    pub mode: String,
    pub key: String,
    pub action: String,
}

/// Editor state exposed to Lua scripts
#[derive(Debug, Clone, Default)]
pub struct LuaEditorState {
    pub current_file: Option<String>,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub mode: String,
    pub options: HashMap<String, LuaOptionValue>,
}

/// Option values that can be set via Lua
#[derive(Debug, Clone)]
pub enum LuaOptionValue {
    Bool(bool),
    Int(i64),
    String(String),
}

impl LuaEngine {
    /// Create a new Lua scripting engine
    pub fn new() -> LuaResult<Self> {
        let lua = Lua::new();
        let custom_commands = Arc::new(Mutex::new(HashMap::new()));
        let custom_keymaps = Arc::new(Mutex::new(Vec::new()));
        let messages = Arc::new(Mutex::new(Vec::new()));

        let engine = Self {
            lua,
            custom_commands,
            custom_keymaps,
            messages,
        };

        engine.setup_api()?;
        Ok(engine)
    }

    /// Setup the quirks API table for Lua scripts
    fn setup_api(&self) -> LuaResult<()> {
        let globals = self.lua.globals();

        // Create the main 'quirks' table
        let quirks = self.lua.create_table()?;

        // quirks.print(msg) - Print a message to the status line
        let messages = Arc::clone(&self.messages);
        let print_fn = self.lua.create_function(move |_, msg: String| {
            if let Ok(mut msgs) = messages.lock() {
                msgs.push(msg);
            }
            Ok(())
        })?;
        quirks.set("print", print_fn)?;

        // quirks.command(name, callback) - Register a custom command
        let commands = Arc::clone(&self.custom_commands);
        let command_fn = self.lua.create_function(move |_, (name, action): (String, String)| {
            if let Ok(mut cmds) = commands.lock() {
                cmds.insert(name, action);
            }
            Ok(())
        })?;
        quirks.set("command", command_fn)?;

        // quirks.keymap(mode, key, action) - Register a keybinding
        let keymaps = Arc::clone(&self.custom_keymaps);
        let keymap_fn = self.lua.create_function(move |_, (mode, key, action): (String, String, String)| {
            if let Ok(mut maps) = keymaps.lock() {
                maps.push(Keymap { mode, key, action });
            }
            Ok(())
        })?;
        quirks.set("keymap", keymap_fn)?;

        // quirks.set_option(name, value) - Set an editor option
        let set_option_fn = self.lua.create_function(|_, (_name, _value): (String, Value)| {
            // Options will be collected and applied by the editor
            // For now, just acknowledge the call
            Ok(())
        })?;
        quirks.set("set_option", set_option_fn)?;

        // quirks.get_option(name) - Get an editor option
        let get_option_fn = self.lua.create_function(|_, _name: String| {
            // Return nil for now, editor will provide real values
            Ok(Value::Nil)
        })?;
        quirks.set("get_option", get_option_fn)?;

        // quirks.version - Editor version
        quirks.set("version", env!("CARGO_PKG_VERSION"))?;

        // quirks.api_version - API version for compatibility checking
        quirks.set("api_version", 1)?;

        globals.set("quirks", quirks)?;

        // Also provide vim-compatible 'vim' table for familiarity
        let vim = self.lua.create_table()?;
        let vim_cmd_fn = self.lua.create_function(|_, _cmd: String| {
            // Execute a vim-style command
            Ok(())
        })?;
        vim.set("cmd", vim_cmd_fn)?;
        globals.set("vim", vim)?;

        Ok(())
    }

    /// Load and execute the user's init.lua
    pub fn load_user_config(&self) -> LuaResult<()> {
        let config_path = Self::get_config_path();
        
        if config_path.exists() {
            let script = std::fs::read_to_string(&config_path)
                .map_err(|e| LuaError::Config(format!("Failed to read {}: {}", config_path.display(), e)))?;
            
            self.execute(&script)
                .map_err(|e| LuaError::Config(format!("Error in {}: {}", config_path.display(), e)))?;
        }

        Ok(())
    }

    /// Get the path to the user's init.lua
    fn get_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("quirks")
            .join("init.lua")
    }

    /// Execute a Lua script
    pub fn execute(&self, script: &str) -> LuaResult<()> {
        self.lua.load(script).exec()?;
        Ok(())
    }

    /// Execute a Lua script and return a value
    pub fn eval<T: mlua::FromLua>(&self, script: &str) -> LuaResult<T> {
        let result = self.lua.load(script).eval()?;
        Ok(result)
    }

    /// Get all registered custom commands
    pub fn get_custom_commands(&self) -> HashMap<String, String> {
        self.custom_commands
            .lock()
            .map(|cmds| cmds.clone())
            .unwrap_or_default()
    }

    /// Get all registered keymaps
    pub fn get_custom_keymaps(&self) -> Vec<Keymap> {
        self.custom_keymaps
            .lock()
            .map(|maps| maps.clone())
            .unwrap_or_default()
    }

    /// Get and clear pending messages
    pub fn take_messages(&self) -> Vec<String> {
        self.messages
            .lock()
            .map(|mut msgs| std::mem::take(&mut *msgs))
            .unwrap_or_default()
    }

    /// Check if a custom command exists
    pub fn has_command(&self, name: &str) -> bool {
        self.custom_commands
            .lock()
            .map(|cmds| cmds.contains_key(name))
            .unwrap_or(false)
    }

    /// Execute a custom command by name
    pub fn run_command(&self, name: &str) -> LuaResult<()> {
        let action = self
            .custom_commands
            .lock()
            .map(|cmds| cmds.get(name).cloned())
            .unwrap_or(None);

        if let Some(action) = action {
            self.execute(&action)?;
        }

        Ok(())
    }
}

impl Default for LuaEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create Lua engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_engine() {
        let engine = LuaEngine::new().unwrap();
        assert!(engine.get_custom_commands().is_empty());
    }

    #[test]
    fn test_execute_script() {
        let engine = LuaEngine::new().unwrap();
        engine.execute("local x = 1 + 1").unwrap();
    }

    #[test]
    fn test_quirks_print() {
        let engine = LuaEngine::new().unwrap();
        engine.execute(r#"quirks.print("Hello, Quirks!")"#).unwrap();
        let messages = engine.take_messages();
        assert_eq!(messages, vec!["Hello, Quirks!"]);
    }

    #[test]
    fn test_register_keymap() {
        let engine = LuaEngine::new().unwrap();
        engine.execute(r#"quirks.keymap("n", "<leader>w", ":w<CR>")"#).unwrap();
        let keymaps = engine.get_custom_keymaps();
        assert_eq!(keymaps.len(), 1);
        assert_eq!(keymaps[0].mode, "n");
        assert_eq!(keymaps[0].key, "<leader>w");
    }

    #[test]
    fn test_register_command() {
        let engine = LuaEngine::new().unwrap();
        engine.execute(r#"quirks.command("greet", "quirks.print('Hi!')")"#).unwrap();
        assert!(engine.has_command("greet"));
    }

    #[test]
    fn test_version() {
        let engine = LuaEngine::new().unwrap();
        let version: String = engine.eval("return quirks.version").unwrap();
        assert!(!version.is_empty());
    }
}
