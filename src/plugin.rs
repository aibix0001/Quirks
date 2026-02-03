//! Plugin architecture for Quirks
//!
//! Provides a simple plugin system with hooks and events.

use std::collections::HashMap;
use std::path::PathBuf;

/// Events that plugins can listen to
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Event {
    /// Editor started
    EditorStart,
    /// Editor about to quit
    EditorQuit,
    /// File opened
    FileOpen(PathBuf),
    /// File saved
    FileSave(PathBuf),
    /// File closed
    FileClose(PathBuf),
    /// Buffer changed
    BufferChange,
    /// Cursor moved
    CursorMove,
    /// Mode changed
    ModeChange(String),
    /// Command executed
    Command(String),
    /// Key pressed (for custom keybindings)
    KeyPress(String),
}

/// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin author
    pub author: String,
    /// Plugin description
    pub description: String,
    /// Plugin homepage/repository
    pub homepage: Option<String>,
}

impl PluginInfo {
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            author: String::new(),
            description: String::new(),
            homepage: None,
        }
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = author.to_string();
        self
    }

    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    pub fn with_homepage(mut self, url: &str) -> Self {
        self.homepage = Some(url.to_string());
        self
    }
}

/// Result of a plugin action
#[derive(Debug, Clone)]
pub enum PluginResult {
    /// Action completed successfully
    Ok,
    /// Action completed with a message
    Message(String),
    /// Action failed with an error
    Error(String),
    /// Request the editor to execute a command
    Command(String),
    /// Request to modify the buffer
    BufferEdit { line: usize, col: usize, text: String },
    /// Continue to next handler (don't stop event propagation)
    Continue,
}

/// Trait that plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin information
    fn info(&self) -> PluginInfo;
    
    /// Called when the plugin is loaded
    fn on_load(&mut self) -> PluginResult {
        PluginResult::Ok
    }
    
    /// Called when the plugin is unloaded
    fn on_unload(&mut self) -> PluginResult {
        PluginResult::Ok
    }
    
    /// Handle an event
    fn on_event(&mut self, event: &Event) -> PluginResult {
        PluginResult::Continue
    }
    
    /// Get the events this plugin wants to receive
    fn subscribed_events(&self) -> Vec<Event> {
        Vec::new()
    }
}

/// Plugin manager
pub struct PluginManager {
    /// Loaded plugins
    plugins: Vec<Box<dyn Plugin>>,
    /// Plugin load order
    load_order: Vec<String>,
    /// Disabled plugins
    disabled: Vec<String>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            load_order: Vec::new(),
            disabled: Vec::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> PluginResult {
        let info = plugin.info();
        let name = info.name.clone();
        
        // Check if already loaded
        if self.load_order.contains(&name) {
            return PluginResult::Error(format!("Plugin '{}' already loaded", name));
        }
        
        self.plugins.push(plugin);
        self.load_order.push(name.clone());
        
        // Call on_load for the new plugin
        if let Some(plugin) = self.plugins.last_mut() {
            plugin.on_load()
        } else {
            PluginResult::Ok
        }
    }

    /// Unload a plugin by name
    pub fn unload(&mut self, name: &str) -> PluginResult {
        let idx = self.load_order.iter().position(|n| n == name);
        
        if let Some(idx) = idx {
            let result = self.plugins[idx].on_unload();
            self.plugins.remove(idx);
            self.load_order.remove(idx);
            result
        } else {
            PluginResult::Error(format!("Plugin '{}' not found", name))
        }
    }

    /// Disable a plugin (keep loaded but don't call handlers)
    pub fn disable(&mut self, name: &str) {
        if !self.disabled.contains(&name.to_string()) {
            self.disabled.push(name.to_string());
        }
    }

    /// Enable a disabled plugin
    pub fn enable(&mut self, name: &str) {
        self.disabled.retain(|n| n != name);
    }

    /// Check if a plugin is enabled
    pub fn is_enabled(&self, name: &str) -> bool {
        self.load_order.contains(&name.to_string()) && !self.disabled.contains(&name.to_string())
    }

    /// Dispatch an event to all plugins
    pub fn dispatch(&mut self, event: &Event) -> Vec<PluginResult> {
        let mut results = Vec::new();
        
        for (idx, plugin) in self.plugins.iter_mut().enumerate() {
            let name = &self.load_order[idx];
            
            // Skip disabled plugins
            if self.disabled.contains(name) {
                continue;
            }
            
            // Check if plugin is subscribed to this event
            let subscribed = plugin.subscribed_events();
            let should_handle = subscribed.is_empty() || subscribed.contains(event);
            
            if should_handle {
                let result = plugin.on_event(event);
                
                // Stop propagation unless Continue
                match &result {
                    PluginResult::Continue => {}
                    _ => {
                        results.push(result);
                        // Could add option to stop propagation here
                    }
                }
            }
        }
        
        results
    }

    /// Get list of loaded plugins
    pub fn list(&self) -> Vec<PluginInfo> {
        self.plugins.iter().map(|p| p.info()).collect()
    }

    /// Get plugin count
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

// Example built-in plugins

/// Auto-save plugin
pub struct AutoSavePlugin {
    interval_ms: u64,
    enabled: bool,
}

impl AutoSavePlugin {
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval_ms,
            enabled: true,
        }
    }
}

impl Plugin for AutoSavePlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("autosave", "1.0.0")
            .with_author("Quirks Team")
            .with_description("Automatically saves files at regular intervals")
    }

    fn on_event(&mut self, event: &Event) -> PluginResult {
        if !self.enabled {
            return PluginResult::Continue;
        }
        
        match event {
            Event::BufferChange => {
                // In a real implementation, this would set up a timer
                PluginResult::Continue
            }
            _ => PluginResult::Continue,
        }
    }

    fn subscribed_events(&self) -> Vec<Event> {
        vec![Event::BufferChange]
    }
}

/// Trailing whitespace trimmer
pub struct TrimWhitespacePlugin {
    trim_on_save: bool,
}

impl TrimWhitespacePlugin {
    pub fn new() -> Self {
        Self { trim_on_save: true }
    }
}

impl Default for TrimWhitespacePlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for TrimWhitespacePlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("trim-whitespace", "1.0.0")
            .with_author("Quirks Team")
            .with_description("Trims trailing whitespace on save")
    }

    fn on_event(&mut self, event: &Event) -> PluginResult {
        match event {
            Event::FileSave(_path) if self.trim_on_save => {
                // Signal that we want to trim whitespace
                PluginResult::Command("trim-trailing-whitespace".to_string())
            }
            _ => PluginResult::Continue,
        }
    }

    fn subscribed_events(&self) -> Vec<Event> {
        vec![Event::FileSave(PathBuf::new())]
    }
}

/// File backup plugin
pub struct BackupPlugin {
    backup_dir: Option<PathBuf>,
}

impl BackupPlugin {
    pub fn new() -> Self {
        Self { backup_dir: None }
    }

    pub fn with_backup_dir(mut self, dir: PathBuf) -> Self {
        self.backup_dir = Some(dir);
        self
    }
}

impl Default for BackupPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for BackupPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::new("backup", "1.0.0")
            .with_author("Quirks Team")
            .with_description("Creates backup copies of files before saving")
    }

    fn on_event(&mut self, event: &Event) -> PluginResult {
        match event {
            Event::FileSave(path) => {
                // In a real implementation, this would create a backup
                PluginResult::Message(format!("Would backup: {:?}", path))
            }
            _ => PluginResult::Continue,
        }
    }

    fn subscribed_events(&self) -> Vec<Event> {
        vec![Event::FileSave(PathBuf::new())]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestPlugin {
        name: String,
        call_count: usize,
    }

    impl TestPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                call_count: 0,
            }
        }
    }

    impl Plugin for TestPlugin {
        fn info(&self) -> PluginInfo {
            PluginInfo::new(&self.name, "1.0.0")
        }

        fn on_event(&mut self, _event: &Event) -> PluginResult {
            self.call_count += 1;
            PluginResult::Ok
        }
    }

    #[test]
    fn test_plugin_manager_register() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(TestPlugin::new("test"));
        
        let result = manager.register(plugin);
        assert!(matches!(result, PluginResult::Ok));
        assert_eq!(manager.count(), 1);
    }

    #[test]
    fn test_plugin_manager_list() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("plugin1")));
        manager.register(Box::new(TestPlugin::new("plugin2")));
        
        let list = manager.list();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name, "plugin1");
        assert_eq!(list[1].name, "plugin2");
    }

    #[test]
    fn test_plugin_manager_disable() {
        let mut manager = PluginManager::new();
        manager.register(Box::new(TestPlugin::new("test")));
        
        assert!(manager.is_enabled("test"));
        manager.disable("test");
        assert!(!manager.is_enabled("test"));
        manager.enable("test");
        assert!(manager.is_enabled("test"));
    }

    #[test]
    fn test_plugin_info_builder() {
        let info = PluginInfo::new("test", "1.0.0")
            .with_author("Author")
            .with_description("Description")
            .with_homepage("https://example.com");
        
        assert_eq!(info.name, "test");
        assert_eq!(info.author, "Author");
        assert_eq!(info.description, "Description");
        assert_eq!(info.homepage, Some("https://example.com".to_string()));
    }
}
