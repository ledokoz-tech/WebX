// Tab Management System
use crate::core::{Tab, BrowserState};
use std::sync::{Arc, Mutex};

/// Tab manager for handling multiple tabs
pub struct TabManager {
    state: Arc<Mutex<BrowserState>>,
}

impl TabManager {
    /// Create a new tab manager
    pub fn new(state: Arc<Mutex<BrowserState>>) -> Self {
        Self { state }
    }

    /// Create a new tab
    pub fn create_tab(&self, url: Option<String>) -> usize {
        let mut state = self.state.lock().unwrap();
        let tab_url = url.unwrap_or_else(|| state.settings.home_page.clone());
        state.add_tab(tab_url)
    }

    /// Close a tab
    pub fn close_tab(&self, tab_id: usize) -> bool {
        let mut state = self.state.lock().unwrap();
        
        if state.tabs.contains_key(&tab_id) {
            state.remove_tab(tab_id);
            true
        } else {
            false
        }
    }

    /// Switch to a specific tab
    pub fn switch_to_tab(&self, tab_id: usize) -> bool {
        let mut state = self.state.lock().unwrap();
        
        if state.tabs.contains_key(&tab_id) {
            state.active_tab_id = Some(tab_id);
            true
        } else {
            false
        }
    }

    /// Get all tabs
    pub fn get_tabs(&self) -> Vec<Tab> {
        let state = self.state.lock().unwrap();
        state.tabs.values().cloned().collect()
    }

    /// Get active tab
    pub fn get_active_tab(&self) -> Option<Tab> {
        let state = self.state.lock().unwrap();
        state.active_tab().cloned()
    }

    /// Duplicate current tab
    pub fn duplicate_tab(&self) -> Option<usize> {
        let state = self.state.lock().unwrap();
        if let Some(active_tab) = state.active_tab() {
            let new_tab_id = state.next_tab_id;
            drop(state); // Release lock before calling create_tab
            
            Some(self.create_tab(Some(active_tab.url.clone())))
        } else {
            None
        }
    }

    /// Move tab to a new position
    pub fn move_tab(&self, tab_id: usize, new_index: usize) -> bool {
        // This would require changing the data structure to support ordering
        // For now, we'll just log that this is a planned feature
        tracing::info!("Moving tab {} to index {}", tab_id, new_index);
        true
    }

    /// Pin/unpin a tab
    pub fn pin_tab(&self, tab_id: usize, pinned: bool) -> bool {
        // Would need to add pinned field to Tab struct
        tracing::info!("Setting tab {} pinned state to {}", tab_id, pinned);
        true
    }

    /// Mute/unmute a tab
    pub fn mute_tab(&self, tab_id: usize, muted: bool) -> bool {
        // Would need to add muted field to Tab struct
        tracing::info!("Setting tab {} muted state to {}", tab_id, muted);
        true
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        let state = self.state.lock().unwrap();
        state.tabs.len()
    }

    /// Check if tab exists
    pub fn tab_exists(&self, tab_id: usize) -> bool {
        let state = self.state.lock().unwrap();
        state.tabs.contains_key(&tab_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::BrowserSettings;

    #[test]
    fn test_create_and_close_tab() {
        let mut state = BrowserState::new();
        state.settings = BrowserSettings::default();
        let state_arc = Arc::new(Mutex::new(state));
        
        let manager = TabManager::new(state_arc.clone());
        
        // Create a tab
        let tab_id = manager.create_tab(None);
        assert!(tab_id > 0);
        assert_eq!(manager.tab_count(), 1);
        
        // Close the tab
        assert!(manager.close_tab(tab_id));
        assert_eq!(manager.tab_count(), 0);
    }

    #[test]
    fn test_switch_tabs() {
        let mut state = BrowserState::new();
        state.settings = BrowserSettings::default();
        let state_arc = Arc::new(Mutex::new(state));
        
        let manager = TabManager::new(state_arc.clone());
        
        let tab1 = manager.create_tab(Some("https://example.com".to_string()));
        let tab2 = manager.create_tab(Some("https://google.com".to_string()));
        
        assert_eq!(manager.tab_count(), 2);
        
        // Switch to tab2
        assert!(manager.switch_to_tab(tab2));
        let active = manager.get_active_tab().unwrap();
        assert_eq!(active.id, tab2);
    }
}