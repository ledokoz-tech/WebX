// Dark Mode Implementation
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Dark mode preferences
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DarkModePreference {
    Light,
    Dark,
    Auto,
}

/// Current theme state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeState {
    pub current_mode: DarkModePreference,
    pub is_dark: bool,
    pub system_is_dark: bool,
}

impl Default for ThemeState {
    fn default() -> Self {
        Self {
            current_mode: DarkModePreference::Auto,
            is_dark: false,
            system_is_dark: false,
        }
    }
}

/// Dark mode manager
pub struct DarkModeManager {
    state: Arc<Mutex<ThemeState>>,
    config_path: std::path::PathBuf,
}

impl DarkModeManager {
    /// Create a new dark mode manager
    pub fn new(config_dir: Option<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
            path.push("webx");
            path.push("themes");
            path
        });
        
        // Create config directory
        std::fs::create_dir_all(&config_dir)?;
        
        let config_path = config_dir.join("theme.json");
        
        let state = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            ThemeState::default()
        };
        
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            config_path,
        })
    }

    /// Set dark mode preference
    pub fn set_preference(&self, preference: DarkModePreference) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().unwrap();
        state.current_mode = preference.clone();
        state.is_dark = self.should_be_dark(&preference, state.system_is_dark);
        
        // Save to config
        let content = serde_json::to_string_pretty(&*state)?;
        std::fs::write(&self.config_path, content)?;
        
        Ok(())
    }

    /// Toggle between light and dark mode
    pub fn toggle_mode(&self) -> Result<DarkModePreference, Box<dyn std::error::Error>> {
        let current = self.get_current_preference();
        let new_preference = match current {
            DarkModePreference::Light => DarkModePreference::Dark,
            DarkModePreference::Dark => DarkModePreference::Light,
            DarkModePreference::Auto => DarkModePreference::Dark, // Toggle to dark when in auto
        };
        
        self.set_preference(new_preference.clone())?;
        Ok(new_preference)
    }

    /// Update system theme detection
    pub fn update_system_theme(&self, is_system_dark: bool) -> Result<(), Box<dyn std::error::Error>> {
        let mut state = self.state.lock().unwrap();
        state.system_is_dark = is_system_dark;
        state.is_dark = self.should_be_dark(&state.current_mode, is_system_dark);
        
        // Save to config
        let content = serde_json::to_string_pretty(&*state)?;
        std::fs::write(&self.config_path, content)?;
        
        Ok(())
    }

    /// Get current theme state
    pub fn get_theme_state(&self) -> ThemeState {
        self.state.lock().unwrap().clone()
    }

    /// Get current preference
    pub fn get_current_preference(&self) -> DarkModePreference {
        self.state.lock().unwrap().current_mode.clone()
    }

    /// Check if dark mode is currently active
    pub fn is_dark_mode(&self) -> bool {
        self.state.lock().unwrap().is_dark
    }

    /// Get CSS variables for the current theme
    pub fn get_css_variables(&self) -> String {
        if self.is_dark_mode() {
            self.get_dark_css()
        } else {
            self.get_light_css()
        }
    }

    /// Get injected JavaScript for theme switching
    pub fn get_theme_script(&self) -> String {
        format!(
            r#"
(function() {{
    const isDark = {};
    const root = document.documentElement;
    
    if (isDark) {{
        root.classList.add('dark-theme');
        root.classList.remove('light-theme');
    }} else {{
        root.classList.add('light-theme');
        root.classList.remove('dark-theme');
    }}
    
    // Listen for theme changes from the browser
    window.addEventListener('webx-theme-change', function(e) {{
        const isDark = e.detail.isDark;
        if (isDark) {{
            root.classList.add('dark-theme');
            root.classList.remove('light-theme');
        }} else {{
            root.classList.add('light-theme');
            root.classList.remove('dark-theme');
        }}
    }});
}})();
"#,
            self.is_dark_mode()
        )
    }

    // Private helper methods
    
    fn should_be_dark(&self, preference: &DarkModePreference, system_is_dark: bool) -> bool {
        match preference {
            DarkModePreference::Light => false,
            DarkModePreference::Dark => true,
            DarkModePreference::Auto => system_is_dark,
        }
    }
    
    fn get_light_css(&self) -> String {
        r#"
:root {
    /* Base colors */
    --bg-primary: #ffffff;
    --bg-secondary: #f8f9fa;
    --bg-tertiary: #e9ecef;
    
    /* Text colors */
    --text-primary: #212529;
    --text-secondary: #6c757d;
    --text-tertiary: #adb5bd;
    
    /* Border colors */
    --border-primary: #dee2e6;
    --border-secondary: #ced4da;
    
    /* Interactive colors */
    --accent-primary: #0d6efd;
    --accent-hover: #0b5ed7;
    --accent-active: #0a58ca;
    
    /* Status colors */
    --success: #198754;
    --warning: #ffc107;
    --error: #dc3545;
    --info: #0dcaf0;
    
    /* Shadows */
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.1);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
    
    /* Scrollbar */
    --scrollbar-thumb: #c1c1c1;
    --scrollbar-track: #f1f1f1;
}

.light-theme {
    color-scheme: light;
}
"#
        .to_string()
    }
    
    fn get_dark_css(&self) -> String {
        r#"
:root {
    /* Base colors */
    --bg-primary: #121212;
    --bg-secondary: #1e1e1e;
    --bg-tertiary: #2d2d2d;
    
    /* Text colors */
    --text-primary: #e0e0e0;
    --text-secondary: #a0a0a0;
    --text-tertiary: #707070;
    
    /* Border colors */
    --border-primary: #333333;
    --border-secondary: #444444;
    
    /* Interactive colors */
    --accent-primary: #4d90fe;
    --accent-hover: #5d9bff;
    --accent-active: #6daaff;
    
    /* Status colors */
    --success: #4caf50;
    --warning: #ff9800;
    --error: #f44336;
    --info: #2196f3;
    
    /* Shadows */
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.3);
    --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.4);
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.5);
    
    /* Scrollbar */
    --scrollbar-thumb: #555555;
    --scrollbar-track: #2d2d2d;
}

.dark-theme {
    color-scheme: dark;
}

/* Ensure proper contrast for common elements */
.dark-theme a {
    color: var(--accent-primary);
}

.dark-theme a:hover {
    color: var(--accent-hover);
}

.dark-theme input,
.dark-theme textarea,
.dark-theme select {
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    border-color: var(--border-primary);
}

.dark-theme button {
    background-color: var(--accent-primary);
    color: white;
}

.dark-theme ::selection {
    background-color: var(--accent-primary);
    color: white;
}

/* Images in dark mode */
.dark-theme img {
    filter: brightness(0.9) contrast(1.1);
}

/* Code blocks */
.dark-theme pre,
.dark-theme code {
    background-color: var(--bg-tertiary);
    color: var(--text-primary);
}
"#
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_dark_mode_preferences() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DarkModeManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test default state
        assert_eq!(manager.get_current_preference(), DarkModePreference::Auto);
        assert!(!manager.is_dark_mode()); // Assuming system is light by default
        
        // Test setting to dark
        manager.set_preference(DarkModePreference::Dark).unwrap();
        assert_eq!(manager.get_current_preference(), DarkModePreference::Dark);
        assert!(manager.is_dark_mode());
        
        // Test toggling
        let new_pref = manager.toggle_mode().unwrap();
        assert_eq!(new_pref, DarkModePreference::Light);
        assert!(!manager.is_dark_mode());
    }

    #[test]
    fn test_system_theme_updates() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DarkModeManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Initially auto mode
        manager.set_preference(DarkModePreference::Auto).unwrap();
        
        // System is dark
        manager.update_system_theme(true).unwrap();
        assert!(manager.is_dark_mode());
        
        // System is light
        manager.update_system_theme(false).unwrap();
        assert!(!manager.is_dark_mode());
    }

    #[test]
    fn test_css_generation() {
        let manager = DarkModeManager::new(None).unwrap();
        
        // Test light mode CSS
        manager.set_preference(DarkModePreference::Light).unwrap();
        let light_css = manager.get_css_variables();
        assert!(light_css.contains("--bg-primary: #ffffff"));
        assert!(light_css.contains(".light-theme"));
        
        // Test dark mode CSS
        manager.set_preference(DarkModePreference::Dark).unwrap();
        let dark_css = manager.get_css_variables();
        assert!(dark_css.contains("--bg-primary: #121212"));
        assert!(dark_css.contains(".dark-theme"));
    }
}