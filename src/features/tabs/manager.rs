// Tab Manager Core Logic
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
        let active_url = {
            let state = self.state.lock().unwrap();
            state.active_tab().map(|tab| tab.url.clone())
        };
        
        if let Some(url) = active_url {
            Some(self.create_tab(Some(url)))
        } else {
            None
        }
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