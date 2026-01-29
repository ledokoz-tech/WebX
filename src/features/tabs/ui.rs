// Tab UI Components
use crate::core::Tab;

/// Visual representation of tabs in the UI
pub struct TabUI {
    tabs: Vec<TabVisual>,
    active_tab_index: Option<usize>,
    max_visible_tabs: usize,
}

/// Visual representation of a single tab
pub struct TabVisual {
    pub tab_id: usize,
    pub title: String,
    pub url: String,
    pub is_active: bool,
    pub is_loading: bool,
    pub favicon: Option<String>,
    pub pinned: bool,
    pub muted: bool,
}

impl TabUI {
    /// Create new tab UI manager
    pub fn new(max_visible_tabs: usize) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: None,
            max_visible_tabs,
        }
    }

    /// Update tab visuals from core tabs
    pub fn update_from_tabs(&mut self, tabs: &[Tab], active_tab_id: Option<usize>) {
        self.tabs.clear();
        
        for tab in tabs {
            let is_active = active_tab_id == Some(tab.id);
            if is_active {
                self.active_tab_index = Some(self.tabs.len());
            }
            
            self.tabs.push(TabVisual {
                tab_id: tab.id,
                title: tab.title.clone(),
                url: tab.url.clone(),
                is_active,
                is_loading: tab.is_loading,
                favicon: tab.favicon.clone(),
                pinned: false, // Would be tracked separately
                muted: false,  // Would be tracked separately
            });
        }
    }

    /// Get visible tabs for rendering
    pub fn get_visible_tabs(&self) -> &[TabVisual] {
        let end = std::cmp::min(self.tabs.len(), self.max_visible_tabs);
        &self.tabs[..end]
    }

    /// Get tab by index
    pub fn get_tab_by_index(&self, index: usize) -> Option<&TabVisual> {
        self.tabs.get(index)
    }

    /// Get active tab visual
    pub fn get_active_tab(&self) -> Option<&TabVisual> {
        self.active_tab_index.and_then(|idx| self.tabs.get(idx))
    }

    /// Check if tab bar needs scrolling
    pub fn needs_scroll(&self) -> bool {
        self.tabs.len() > self.max_visible_tabs
    }

    /// Get tab count
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}