// Core browser types and structures
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Represents a browser tab
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub is_loading: bool,
    pub can_go_back: bool,
    pub can_go_forward: bool,
}

impl Tab {
    /// Create a new tab with the given ID and URL
    pub fn new(id: usize, url: String) -> Self {
        Self {
            id,
            title: "New Tab".to_string(),
            url,
            favicon: None,
            is_loading: false,
            can_go_back: false,
            can_go_forward: false,
        }
    }
}

/// Represents a bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub favicon: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Represents a history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub visited_at: DateTime<Utc>,
}

/// Represents a download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub id: usize,
    pub url: String,
    pub filename: String,
    pub path: String,
    pub size: u64,
    pub downloaded: u64,
    pub status: DownloadStatus,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

/// Browser settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSettings {
    pub home_page: String,
    pub search_engine: SearchEngine,
    pub default_zoom: f64,
    pub enable_javascript: bool,
    pub enable_cookies: bool,
    pub enable_cache: bool,
    pub block_popups: bool,
    pub user_agent: Option<String>,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            home_page: "https://www.google.com".to_string(),
            search_engine: SearchEngine::Google,
            default_zoom: 1.0,
            enable_javascript: true,
            enable_cookies: true,
            enable_cache: true,
            block_popups: true,
            user_agent: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SearchEngine {
    Google,
    DuckDuckGo,
    Bing,
    Brave,
}

impl SearchEngine {
    /// Get the search URL for a query
    pub fn search_url(&self, query: &str) -> String {
        let encoded = urlencoding::encode(query);
        match self {
            SearchEngine::Google => format!("https://www.google.com/search?q={}", encoded),
            SearchEngine::DuckDuckGo => format!("https://duckduckgo.com/?q={}", encoded),
            SearchEngine::Bing => format!("https://www.bing.com/search?q={}", encoded),
            SearchEngine::Brave => format!("https://search.brave.com/search?q={}", encoded),
        }
    }
}

/// Browser state
pub struct BrowserState {
    pub tabs: HashMap<usize, Tab>,
    pub active_tab_id: Option<usize>,
    pub next_tab_id: usize,
    pub bookmarks: Vec<Bookmark>,
    pub history: Vec<HistoryEntry>,
    pub downloads: Vec<Download>,
    pub settings: BrowserSettings,
}

impl BrowserState {
    /// Create a new browser state
    pub fn new() -> Self {
        Self {
            tabs: HashMap::new(),
            active_tab_id: None,
            next_tab_id: 1,
            bookmarks: Vec::new(),
            history: Vec::new(),
            downloads: Vec::new(),
            settings: BrowserSettings::default(),
        }
    }

    /// Add a new tab
    pub fn add_tab(&mut self, url: String) -> usize {
        let id = self.next_tab_id;
        self.next_tab_id += 1;
        
        let tab = Tab::new(id, url);
        self.tabs.insert(id, tab);
        self.active_tab_id = Some(id);
        
        id
    }

    /// Remove a tab
    pub fn remove_tab(&mut self, id: usize) {
        self.tabs.remove(&id);
        
        // If we removed the active tab, switch to another
        if self.active_tab_id == Some(id) {
            self.active_tab_id = self.tabs.keys().next().copied();
        }
    }

    /// Get the active tab
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_tab_id.and_then(|id| self.tabs.get(&id))
    }

    /// Get the active tab mutably
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_tab_id.and_then(|id| self.tabs.get_mut(&id))
    }

    /// Add a bookmark
    pub fn add_bookmark(&mut self, title: String, url: String) {
        let id = self.bookmarks.len() + 1;
        self.bookmarks.push(Bookmark {
            id,
            title,
            url,
            favicon: None,
            created_at: Utc::now(),
        });
    }

    /// Remove a bookmark
    pub fn remove_bookmark(&mut self, id: usize) {
        self.bookmarks.retain(|b| b.id != id);
    }

    /// Check if a URL is bookmarked
    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.iter().any(|b| b.url == url)
    }

    /// Add a history entry
    pub fn add_history(&mut self, title: String, url: String) {
        let id = self.history.len() + 1;
        self.history.push(HistoryEntry {
            id,
            title,
            url,
            visited_at: Utc::now(),
        });
        
        // Keep only last 1000 entries
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
    }

    /// Process URL input (add protocol, handle search)
    pub fn process_url(&self, input: &str) -> String {
        // If it looks like a URL, add https if needed
        if input.contains('.') && !input.contains(' ') {
            if input.starts_with("http://") || input.starts_with("https://") {
                input.to_string()
            } else {
                format!("https://{}", input)
            }
        } else {
            // Treat as search query
            self.settings.search_engine.search_url(input)
        }
    }
}

impl Default for BrowserState {
    fn default() -> Self {
        Self::new()
    }
}

/// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
