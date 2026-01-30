// Unified Theme Manager
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Theme preference options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemePreference {
    Light,
    Dark,
    System,
    Custom(String),
}

/// Theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub current_theme: ThemePreference,
    pub auto_detect_system: bool,
    pub custom_themes_dir: Option<PathBuf>,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            current_theme: ThemePreference::System,
            auto_detect_system: true,
            custom_themes_dir: None,
        }
    }
}

/// Unified theme manager that coordinates all theme functionality
pub struct ThemeManager {
    config: ThemeConfig,
    config_path: PathBuf,
}

impl ThemeManager {
    /// Create new theme manager
    pub fn new(
        config: Option<ThemeConfig>,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("themes");
            path
        });

        // Create config directory
        std::fs::create_dir_all(&config_dir)?;

        let mut manager = Self {
            config,
            config_path: config_dir.join("theme_config.json"),
        };

        // Load existing configuration
        manager.load_config()?;

        Ok(manager)
    }

    /// Set current theme
    pub fn set_theme(&mut self, theme: ThemePreference) -> Result<(), Box<dyn std::error::Error>> {
        self.config.current_theme = theme;
        self.save_config()?;
        Ok(())
    }

    /// Get current theme
    pub fn get_current_theme(&self) -> &ThemePreference {
        &self.config.current_theme
    }

    /// Get CSS variables for current theme
    pub fn get_css_variables(&self) -> String {
        match &self.config.current_theme {
            ThemePreference::Light => self.get_light_theme_css(),
            ThemePreference::Dark => self.get_dark_theme_css(),
            ThemePreference::System => {
                // In a real implementation, this would detect system theme
                self.get_dark_theme_css() // Default to dark
            }
            ThemePreference::Custom(name) => self.get_custom_theme_css(name),
        }
    }

    /// Get theme configuration
    pub fn get_config(&self) -> &ThemeConfig {
        &self.config
    }

    /// Set theme configuration
    pub fn set_config(&mut self, config: ThemeConfig) {
        self.config = config;
    }

    /// List available themes
    pub fn list_available_themes(&self) -> Vec<String> {
        let mut themes = vec!["Light".to_string(), "Dark".to_string()];
        
        // Add custom themes if directory exists
        if let Some(custom_dir) = &self.config.custom_themes_dir {
            if custom_dir.exists() {
                if let Ok(entries) = std::fs::read_dir(custom_dir) {
                    for entry in entries.flatten() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(".json") {
                                themes.push(name.trim_end_matches(".json").to_string());
                            }
                        }
                    }
                }
            }
        }
        
        themes
    }

    // Private helper methods
    
    fn get_light_theme_css(&self) -> String {
        r#"
:root {
    /* Light theme colors */
    --bg-primary: #ffffff;
    --bg-secondary: #f8f9fa;
    --bg-tertiary: #e9ecef;
    --text-primary: #212529;
    --text-secondary: #6c757d;
    --accent-primary: #0d6efd;
    --accent-hover: #0b5ed7;
    --border-primary: #dee2e6;
    --success: #198754;
    --warning: #ffc107;
    --error: #dc3545;
    --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
}
"#
        .to_string()
    }

    fn get_dark_theme_css(&self) -> String {
        r#"
:root {
    /* Dark theme colors */
    --bg-primary: #121212;
    --bg-secondary: #1e1e1e;
    --bg-tertiary: #2d2d2d;
    --text-primary: #e0e0e0;
    --text-secondary: #a0a0a0;
    --accent-primary: #4d90fe;
    --accent-hover: #5d9bff;
    --border-primary: #333333;
    --success: #4caf50;
    --warning: #ff9800;
    --error: #f44336;
    --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.3);
}
"#
        .to_string()
    }

    fn get_custom_theme_css(&self, theme_name: &str) -> String {
        if let Some(custom_dir) = &self.config.custom_themes_dir {
            let theme_path = custom_dir.join(format!("{}.json", theme_name));
            if theme_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&theme_path) {
                    if let Ok(theme_data) = serde_json::from_str::<serde_json::Value>(&content) {
                        return self.generate_css_from_theme_data(&theme_data);
                    }
                }
            }
        }
        self.get_dark_theme_css() // Fallback
    }

    fn generate_css_from_theme_data(&self, _theme_data: &serde_json::Value) -> String {
        // This would convert theme JSON to CSS variables
        // Simplified implementation for now
        self.get_dark_theme_css()
    }

    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }

    fn load_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            let content = std::fs::read_to_string(&self.config_path)?;
            self.config = serde_json::from_str(&content)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_theme_manager_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ThemeManager::new(None, Some(temp_dir.path().to_path_buf())).unwrap();

        // Test setting themes
        manager.set_theme(ThemePreference::Dark).unwrap();
        assert_eq!(manager.get_current_theme(), &ThemePreference::Dark);

        manager.set_theme(ThemePreference::Light).unwrap();
        assert_eq!(manager.get_current_theme(), &ThemePreference::Light);

        // Test CSS generation
        let dark_css = manager.get_css_variables();
        assert!(dark_css.contains("--bg-primary: #121212"));

        // Test available themes
        let themes = manager.list_available_themes();
        assert!(themes.contains(&"Light".to_string()));
        assert!(themes.contains(&"Dark".to_string()));
    }

    #[test]
    fn test_theme_configuration() {
        let mut config = ThemeConfig::default();
        config.current_theme = ThemePreference::Custom("Solarized".to_string());

        let manager = ThemeManager::new(Some(config), None).unwrap();
        let current_config = manager.get_config();

        match &current_config.current_theme {
            ThemePreference::Custom(name) => assert_eq!(name, "Solarized"),
            _ => panic!("Expected custom theme"),
        }
    }
}