// Keyboard Shortcut Customization
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Keyboard modifier keys
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Meta, // Cmd on Mac, Windows key on Windows
}

/// Keyboard event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub key: String,
    pub modifiers: Vec<ModifierKey>,
}

/// Action that can be triggered by a keyboard shortcut
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ActionType {
    // Navigation
    NewTab,
    CloseTab,
    NextTab,
    PreviousTab,
    DuplicateTab,
    ReopenClosedTab,
    
    // Page actions
    Reload,
    ForceReload,
    StopLoading,
    Back,
    Forward,
    Home,
    
    // View actions
    ZoomIn,
    ZoomOut,
    ResetZoom,
    ToggleFullscreen,
    ToggleDevTools,
    
    // Editing
    Copy,
    Cut,
    Paste,
    SelectAll,
    Undo,
    Redo,
    
    // Find and bookmarks
    Find,
    FindNext,
    FindPrevious,
    BookmarkPage,
    ShowBookmarks,
    ShowHistory,
    
    // Window management
    NewWindow,
    CloseWindow,
    Minimize,
    Maximize,
    ToggleMenu,
    
    // Custom actions
    Custom(String),
}

/// Keyboard shortcut mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    pub event: KeyEvent,
    pub action: ActionType,
    pub enabled: bool,
    pub description: String,
}

/// Keyboard shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    pub enable_global_shortcuts: bool,
    pub enable_app_shortcuts: bool,
    pub enable_webview_shortcuts: bool,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        Self {
            enable_global_shortcuts: true,
            enable_app_shortcuts: true,
            enable_webview_shortcuts: true,
        }
    }
}

/// Keyboard shortcut manager
pub struct KeyboardShortcuts {
    shortcuts: Arc<Mutex<HashMap<ActionType, KeyboardShortcut>>>,
    config: ShortcutConfig,
    config_path: PathBuf,
}

impl KeyboardShortcuts {
    /// Create a new keyboard shortcuts manager
    pub fn new(
        config: Option<ShortcutConfig>,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("shortcuts");
            path
        });
        
        // Create config directory
        fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("shortcuts.json");
        
        let manager = Self {
            shortcuts: Arc::new(Mutex::new(HashMap::new())),
            config,
            config_path,
        };
        
        // Load existing shortcuts or initialize defaults
        if config_path.exists() {
            manager.load_shortcuts()?;
        } else {
            manager.initialize_default_shortcuts();
        }
        
        Ok(manager)
    }

    /// Register a keyboard shortcut
    pub fn register_shortcut(
        &self,
        action: ActionType,
        key_event: KeyEvent,
        description: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let shortcut = KeyboardShortcut {
            event: key_event,
            action: action.clone(),
            enabled: true,
            description: description.to_string(),
        };
        
        {
            let mut shortcuts = self.shortcuts.lock().unwrap();
            shortcuts.insert(action, shortcut);
        }
        
        self.save_shortcuts()?;
        Ok(())
    }

    /// Unregister a keyboard shortcut
    pub fn unregister_shortcut(&self, action: &ActionType) -> bool {
        let mut shortcuts = self.shortcuts.lock().unwrap();
        shortcuts.remove(action).is_some()
    }

    /// Enable/disable a shortcut
    pub fn set_shortcut_enabled(&self, action: &ActionType, enabled: bool) -> bool {
        let mut shortcuts = self.shortcuts.lock().unwrap();
        if let Some(shortcut) = shortcuts.get_mut(action) {
            shortcut.enabled = enabled;
            let _ = self.save_shortcuts();
            true
        } else {
            false
        }
    }

    /// Get shortcut for an action
    pub fn get_shortcut(&self, action: &ActionType) -> Option<KeyboardShortcut> {
        let shortcuts = self.shortcuts.lock().unwrap();
        shortcuts.get(action).cloned()
    }

    /// Find action for a key event
    pub fn find_action(&self, key_event: &KeyEvent) -> Option<ActionType> {
        let shortcuts = self.shortcuts.lock().unwrap();
        for (action, shortcut) in shortcuts.iter() {
            if shortcut.enabled && &shortcut.event == key_event {
                return Some(action.clone());
            }
        }
        None
    }

    /// Get all registered shortcuts
    pub fn get_all_shortcuts(&self) -> Vec<KeyboardShortcut> {
        let shortcuts = self.shortcuts.lock().unwrap();
        shortcuts.values().cloned().collect()
    }

    /// Reset to default shortcuts
    pub fn reset_to_defaults(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.shortcuts.lock().unwrap().clear();
        self.initialize_default_shortcuts();
        self.save_shortcuts()?;
        Ok(())
    }

    /// Import shortcuts from JSON
    pub fn import_shortcuts(&self, json_data: &str) -> Result<(), Box<dyn std::error::Error>> {
        let shortcuts: Vec<KeyboardShortcut> = serde_json::from_str(json_data)?;
        
        {
            let mut shortcut_map = self.shortcuts.lock().unwrap();
            shortcut_map.clear();
            
            for shortcut in shortcuts {
                shortcut_map.insert(shortcut.action.clone(), shortcut);
            }
        }
        
        self.save_shortcuts()?;
        Ok(())
    }

    /// Export shortcuts to JSON
    pub fn export_shortcuts(&self) -> Result<String, Box<dyn std::error::Error>> {
        let shortcuts = self.get_all_shortcuts();
        Ok(serde_json::to_string_pretty(&shortcuts)?)
    }

    /// Get JavaScript for webview shortcut handling
    pub fn get_webview_shortcut_script(&self) -> String {
        let shortcuts = self.get_all_shortcuts();
        let mut shortcut_map = String::new();
        
        for shortcut in shortcuts {
            if shortcut.enabled {
                let key_combo = self.format_key_combo(&shortcut.event);
                shortcut_map.push_str(&format!(
                    "'{}': '{}',\n",
                    key_combo,
                    self.action_to_js_event(&shortcut.action)
                ));
            }
        }
        
        format!(
            r#"
(function() {{
    const shortcuts = {{
        {}
    }};
    
    document.addEventListener('keydown', function(e) {{
        const keyCombo = getKeyCombo(e);
        if (shortcuts[keyCombo]) {{
            e.preventDefault();
            window.ipc.send({{
                type: 'shortcut-triggered',
                action: shortcuts[keyCombo]
            }});
        }
    }});
    
    function getKeyCombo(e) {{
        let combo = '';
        if (e.ctrlKey) combo += 'Ctrl+';
        if (e.altKey) combo += 'Alt+';
        if (e.shiftKey) combo += 'Shift+';
        if (e.metaKey) combo += 'Meta+';
        combo += e.key.toLowerCase();
        return combo;
    }}
}})();
"#,
            shortcut_map
        )
    }

    /// Set configuration
    pub fn set_config(&mut self, config: ShortcutConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ShortcutConfig {
        &self.config
    }

    // Private helper methods
    
    fn initialize_default_shortcuts(&self) {
        let defaults = vec![
            (ActionType::NewTab, "Ctrl+T", "Open new tab"),
            (ActionType::CloseTab, "Ctrl+W", "Close current tab"),
            (ActionType::NextTab, "Ctrl+Tab", "Switch to next tab"),
            (ActionType::PreviousTab, "Ctrl+Shift+Tab", "Switch to previous tab"),
            (ActionType::Reload, "Ctrl+R", "Reload current page"),
            (ActionType::ForceReload, "Ctrl+Shift+R", "Force reload"),
            (ActionType::Back, "Alt+Left", "Go back"),
            (ActionType::Forward, "Alt+Right", "Go forward"),
            (ActionType::Home, "Alt+Home", "Go to home page"),
            (ActionType::ZoomIn, "Ctrl+Plus", "Zoom in"),
            (ActionType::ZoomOut, "Ctrl+Minus", "Zoom out"),
            (ActionType::ResetZoom, "Ctrl+0", "Reset zoom"),
            (ActionType::Find, "Ctrl+F", "Find in page"),
            (ActionType::BookmarkPage, "Ctrl+D", "Bookmark current page"),
            (ActionType::ShowBookmarks, "Ctrl+Shift+B", "Show bookmarks"),
            (ActionType::ShowHistory, "Ctrl+H", "Show history"),
            (ActionType::ToggleDevTools, "F12", "Toggle developer tools"),
            (ActionType::Copy, "Ctrl+C", "Copy selected text"),
            (ActionType::Cut, "Ctrl+X", "Cut selected text"),
            (ActionType::Paste, "Ctrl+V", "Paste from clipboard"),
            (ActionType::SelectAll, "Ctrl+A", "Select all"),
            (ActionType::Undo, "Ctrl+Z", "Undo last action"),
            (ActionType::Redo, "Ctrl+Y", "Redo last action"),
        ];
        
        let mut shortcuts = self.shortcuts.lock().unwrap();
        
        for (action, key_combo, description) in defaults {
            if let Some(key_event) = self.parse_key_combo(key_combo) {
                let shortcut = KeyboardShortcut {
                    event: key_event,
                    action,
                    enabled: true,
                    description: description.to_string(),
                };
                shortcuts.insert(shortcut.action.clone(), shortcut);
            }
        }
    }
    
    fn parse_key_combo(&self, key_combo: &str) -> Option<KeyEvent> {
        let parts: Vec<&str> = key_combo.split('+').collect();
        if parts.is_empty() {
            return None;
        }
        
        let mut modifiers = Vec::new();
        let mut key = String::new();
        
        for part in parts {
            match part.trim().to_uppercase().as_str() {
                "CTRL" => modifiers.push(ModifierKey::Ctrl),
                "ALT" => modifiers.push(ModifierKey::Alt),
                "SHIFT" => modifiers.push(ModifierKey::Shift),
                "META" | "CMD" | "WIN" => modifiers.push(ModifierKey::Meta),
                k => key = k.to_lowercase(),
            }
        }
        
        if key.is_empty() {
            None
        } else {
            Some(KeyEvent { key, modifiers })
        }
    }
    
    fn format_key_combo(&self, event: &KeyEvent) -> String {
        let mut parts = Vec::new();
        
        for modifier in &event.modifiers {
            match modifier {
                ModifierKey::Ctrl => parts.push("Ctrl"),
                ModifierKey::Alt => parts.push("Alt"),
                ModifierKey::Shift => parts.push("Shift"),
                ModifierKey::Meta => parts.push("Meta"),
            }
        }
        
        parts.push(&event.key);
        parts.join("+")
    }
    
    fn action_to_js_event(&self, action: &ActionType) -> String {
        match action {
            ActionType::NewTab => "new-tab",
            ActionType::CloseTab => "close-tab",
            ActionType::NextTab => "next-tab",
            ActionType::PreviousTab => "previous-tab",
            ActionType::Reload => "reload",
            ActionType::Back => "back",
            ActionType::Forward => "forward",
            ActionType::Find => "find",
            ActionType::ZoomIn => "zoom-in",
            ActionType::ZoomOut => "zoom-out",
            ActionType::BookmarkPage => "bookmark",
            ActionType::ToggleDevTools => "toggle-devtools",
            ActionType::Copy => "copy",
            ActionType::Cut => "cut",
            ActionType::Paste => "paste",
            _ => "custom-action",
        }
        .to_string()
    }
    
    fn save_shortcuts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let shortcuts = self.get_all_shortcuts();
        let content = serde_json::to_string_pretty(&shortcuts)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
    
    fn load_shortcuts(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&self.config_path)?;
        let shortcuts: Vec<KeyboardShortcut> = serde_json::from_str(&content)?;
        
        let mut shortcut_map = self.shortcuts.lock().unwrap();
        shortcut_map.clear();
        
        for shortcut in shortcuts {
            shortcut_map.insert(shortcut.action.clone(), shortcut);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_shortcut_registration() {
        let temp_dir = TempDir::new().unwrap();
        let shortcuts = KeyboardShortcuts::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Register a custom shortcut
        let key_event = KeyEvent {
            key: "k".to_string(),
            modifiers: vec![ModifierKey::Ctrl],
        };
        
        shortcuts
            .register_shortcut(ActionType::Custom("test".to_string()), key_event.clone(), "Test shortcut")
            .unwrap();
        
        // Test finding the action
        let found_action = shortcuts.find_action(&key_event).unwrap();
        match found_action {
            ActionType::Custom(name) => assert_eq!(name, "test"),
            _ => panic!("Expected custom action"),
        }
        
        // Test getting shortcut
        let shortcut = shortcuts.get_shortcut(&found_action).unwrap();
        assert_eq!(shortcut.description, "Test shortcut");
        assert!(shortcut.enabled);
    }

    #[test]
    fn test_default_shortcuts() {
        let shortcuts = KeyboardShortcuts::new(None, None).unwrap();
        
        // Test some default shortcuts exist
        let new_tab_shortcut = shortcuts.get_shortcut(&ActionType::NewTab);
        assert!(new_tab_shortcut.is_some());
        assert_eq!(new_tab_shortcut.unwrap().event.key, "t");
        assert!(new_tab_shortcut.unwrap().event.modifiers.contains(&ModifierKey::Ctrl));
        
        let reload_shortcut = shortcuts.get_shortcut(&ActionType::Reload);
        assert!(reload_shortcut.is_some());
        assert_eq!(reload_shortcut.unwrap().event.key, "r");
    }

    #[test]
    fn test_shortcut_modification() {
        let shortcuts = KeyboardShortcuts::new(None, None).unwrap();
        
        // Test disabling a shortcut
        assert!(shortcuts.set_shortcut_enabled(&ActionType::NewTab, false));
        
        let new_tab_shortcut = shortcuts.get_shortcut(&ActionType::NewTab).unwrap();
        assert!(!new_tab_shortcut.enabled);
        
        // Test that disabled shortcuts aren't found
        let key_event = KeyEvent {
            key: "t".to_string(),
            modifiers: vec![ModifierKey::Ctrl],
        };
        assert!(shortcuts.find_action(&key_event).is_none());
        
        // Test re-enabling
        assert!(shortcuts.set_shortcut_enabled(&ActionType::NewTab, true));
        assert!(shortcuts.find_action(&key_event).is_some());
    }

    #[test]
    fn test_shortcut_import_export() {
        let temp_dir = TempDir::new().unwrap();
        let shortcuts = KeyboardShortcuts::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Export current shortcuts
        let exported = shortcuts.export_shortcuts().unwrap();
        assert!(!exported.is_empty());
        assert!(exported.contains("Ctrl+T"));
        
        // Reset to defaults
        shortcuts.reset_to_defaults().unwrap();
        
        // Import the exported shortcuts
        shortcuts.import_shortcuts(&exported).unwrap();
        
        // Verify shortcuts were imported
        let imported_shortcut = shortcuts.get_shortcut(&ActionType::NewTab);
        assert!(imported_shortcut.is_some());
    }
}