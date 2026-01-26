// Browser configuration and persistence
use crate::core::{BrowserSettings, Bookmark, HistoryEntry};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration manager for the browser
pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Result<Self, std::io::Error> {
        let config_dir = if let Some(proj_dirs) = ProjectDirs::from("com", "Ledokoz", "WebX") {
            proj_dirs.config_dir().to_path_buf()
        } else {
            PathBuf::from(".webx")
        };

        // Create config directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;

        Ok(Self { config_dir })
    }

    /// Get the path to the settings file
    fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    /// Get the path to the bookmarks file
    fn bookmarks_path(&self) -> PathBuf {
        self.config_dir.join("bookmarks.json")
    }

    /// Get the path to the history file
    fn history_path(&self) -> PathBuf {
        self.config_dir.join("history.json")
    }

    /// Load settings from disk
    pub fn load_settings(&self) -> BrowserSettings {
        let path = self.settings_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(settings) = serde_json::from_str(&content) {
                    return settings;
                }
            }
        }
        BrowserSettings::default()
    }

    /// Save settings to disk
    pub fn save_settings(&self, settings: &BrowserSettings) -> Result<(), std::io::Error> {
        let path = self.settings_path();
        let content = serde_json::to_string_pretty(settings)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load bookmarks from disk
    pub fn load_bookmarks(&self) -> Vec<Bookmark> {
        let path = self.bookmarks_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(bookmarks) = serde_json::from_str(&content) {
                    return bookmarks;
                }
            }
        }
        Vec::new()
    }

    /// Save bookmarks to disk
    pub fn save_bookmarks(&self, bookmarks: &[Bookmark]) -> Result<(), std::io::Error> {
        let path = self.bookmarks_path();
        let content = serde_json::to_string_pretty(bookmarks)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load history from disk
    pub fn load_history(&self) -> Vec<HistoryEntry> {
        let path = self.history_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(history) = serde_json::from_str(&content) {
                    return history;
                }
            }
        }
        Vec::new()
    }

    /// Save history to disk
    pub fn save_history(&self, history: &[HistoryEntry]) -> Result<(), std::io::Error> {
        let path = self.history_path();
        let content = serde_json::to_string_pretty(history)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Get the config directory path
    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new().expect("Failed to create config manager")
    }
}
