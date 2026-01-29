// Session Restore Functionality
use crate::core::{Tab, BrowserState};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Session data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub tabs: Vec<SessionTab>,
    pub active_tab_index: Option<usize>,
    pub window_position: Option<(i32, i32)>,
    pub window_size: Option<(u32, u32)>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub session_name: Option<String>,
}

/// Tab data for session storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTab {
    pub url: String,
    pub title: String,
    pub scroll_position: Option<(f64, f64)>,
    pub form_data: Option<String>, // Serialized form data
}

/// Session restore configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub auto_save_interval: Duration,
    pub max_sessions: usize,
    pub save_on_exit: bool,
    pub restore_on_start: bool,
    pub backup_sessions: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            auto_save_interval: Duration::from_secs(30),
            max_sessions: 10,
            save_on_exit: true,
            restore_on_start: true,
            backup_sessions: true,
        }
    }
}

/// Session manager for saving and restoring browsing sessions
pub struct SessionRestore {
    config: SessionConfig,
    sessions_dir: PathBuf,
    backup_dir: PathBuf,
    current_session: Arc<Mutex<Option<SessionData>>>,
    save_timer: Option<tokio::task::JoinHandle<()>>,
}

impl SessionRestore {
    /// Create a new session restore manager
    pub fn new(
        config: Option<SessionConfig>,
        data_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        let data_dir = data_dir.unwrap_or_else(|| {
            let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("sessions");
            path
        });
        
        let sessions_dir = data_dir.clone();
        let backup_dir = data_dir.join("backup");
        
        // Create directories
        fs::create_dir_all(&sessions_dir)?;
        fs::create_dir_all(&backup_dir)?;
        
        let manager = Self {
            config,
            sessions_dir,
            backup_dir,
            current_session: Arc::new(Mutex::new(None)),
            save_timer: None,
        };
        
        Ok(manager)
    }

    /// Capture current browser state as a session
    pub fn capture_session(
        &self,
        browser_state: &BrowserState,
        window_position: Option<(i32, i32)>,
        window_size: Option<(u32, u32)>,
    ) -> SessionData {
        let tabs: Vec<SessionTab> = browser_state
            .tabs
            .values()
            .map(|tab| SessionTab {
                url: tab.url.clone(),
                title: tab.title.clone(),
                scroll_position: None, // Would capture actual scroll position
                form_data: None,       // Would capture form data
            })
            .collect();
        
        let active_tab_index = browser_state
            .active_tab_id
            .and_then(|id| {
                browser_state
                    .tabs
                    .keys()
                    .position(|&tab_id| tab_id == id)
            });
        
        SessionData {
            tabs,
            active_tab_index,
            window_position,
            window_size,
            timestamp: chrono::Utc::now(),
            session_name: None,
        }
    }

    /// Save session to disk
    pub fn save_session(
        &self,
        mut session: SessionData,
        session_name: Option<String>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let session_id = format!("session_{}", uuid::Uuid::new_v4());
        session.session_name = session_name.or_else(|| {
            Some(format!(
                "Session {}",
                session.timestamp.format("%Y-%m-%d %H:%M")
            ))
        });
        
        let filename = format!("{}.json", session_id);
        let path = self.sessions_dir.join(&filename);
        
        let content = serde_json::to_string_pretty(&session)?;
        fs::write(&path, content)?;
        
        // Backup the session
        if self.config.backup_sessions {
            let backup_path = self.backup_dir.join(&filename);
            fs::copy(&path, &backup_path)?;
        }
        
        // Update current session
        *self.current_session.lock().unwrap() = Some(session);
        
        // Clean up old sessions
        self.cleanup_old_sessions()?;
        
        Ok(session_id)
    }

    /// Restore session from disk
    pub fn restore_session(
        &self,
        session_id: &str,
    ) -> Result<SessionData, Box<dyn std::error::Error>> {
        let filename = format!("{}.json", session_id);
        let path = self.sessions_dir.join(&filename);
        
        if !path.exists() {
            return Err("Session not found".into());
        }
        
        let content = fs::read_to_string(&path)?;
        let session: SessionData = serde_json::from_str(&content)?;
        
        Ok(session)
    }

    /// Get list of available sessions
    pub fn list_sessions(&self) -> Result<Vec<(String, SessionData)>, Box<dyn std::error::Error>> {
        let mut sessions = Vec::new();
        
        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                if let Some(filename) = path.file_stem().and_then(|stem| stem.to_str()) {
                    let content = fs::read_to_string(&path)?;
                    if let Ok(session) = serde_json::from_str::<SessionData>(&content) {
                        sessions.push((filename.to_string(), session));
                    }
                }
            }
        }
        
        // Sort by timestamp (newest first)
        sessions.sort_by(|a, b| b.1.timestamp.cmp(&a.1.timestamp));
        
        Ok(sessions)
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let filename = format!("{}.json", session_id);
        let path = self.sessions_dir.join(&filename);
        
        if path.exists() {
            fs::remove_file(&path)?;
        }
        
        // Also remove backup
        let backup_path = self.backup_dir.join(&filename);
        if backup_path.exists() {
            fs::remove_file(&backup_path)?;
        }
        
        Ok(())
    }

    /// Restore browser state from session data
    pub fn apply_session_to_browser(
        &self,
        session: &SessionData,
        browser_state: &mut BrowserState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Clear existing tabs
        browser_state.tabs.clear();
        browser_state.active_tab_id = None;
        
        // Restore tabs
        for (index, session_tab) in session.tabs.iter().enumerate() {
            let tab_id = browser_state.next_tab_id;
            browser_state.next_tab_id += 1;
            
            let tab = Tab {
                id: tab_id,
                title: session_tab.title.clone(),
                url: session_tab.url.clone(),
                favicon: None,
                is_loading: false,
                can_go_back: false,
                can_go_forward: false,
            };
            
            browser_state.tabs.insert(tab_id, tab);
            
            // Set active tab
            if session.active_tab_index == Some(index) {
                browser_state.active_tab_id = Some(tab_id);
            }
        }
        
        // If no active tab was set, activate the first one
        if browser_state.active_tab_id.is_none() {
            browser_state.active_tab_id = browser_state.tabs.keys().next().copied();
        }
        
        Ok(())
    }

    /// Start auto-save timer
    pub fn start_auto_save(
        &mut self,
        browser_state: Arc<Mutex<BrowserState>>,
        get_window_info: impl Fn() -> Option<((i32, i32), (u32, u32))> + Send + 'static,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.save_timer.is_some() {
            self.stop_auto_save();
        }
        
        let interval = self.config.auto_save_interval;
        let sessions_dir = self.sessions_dir.clone();
        let current_session = self.current_session.clone();
        
        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);
            
            loop {
                interval_timer.tick().await;
                
                // Capture and save session
                if let Ok(state) = browser_state.lock() {
                    if let Some((pos, size)) = get_window_info() {
                        let session = SessionRestore::capture_session_static(
                            &state,
                            Some(pos),
                            Some(size),
                        );
                        
                        // Save to temporary file
                        if let Ok(content) = serde_json::to_string(&session) {
                            let _ = fs::write(sessions_dir.join("autosave.json"), content);
                        }
                    }
                }
            }
        });
        
        self.save_timer = Some(handle);
        Ok(())
    }

    /// Stop auto-save timer
    pub fn stop_auto_save(&mut self) {
        if let Some(handle) = self.save_timer.take() {
            handle.abort();
        }
    }

    /// Get last auto-saved session
    pub fn get_last_autosave(&self) -> Option<SessionData> {
        let path = self.sessions_dir.join("autosave.json");
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str(&content) {
                    return Some(session);
                }
            }
        }
        None
    }

    /// Set configuration
    pub fn set_config(&mut self, config: SessionConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &SessionConfig {
        &self.config
    }

    // Private helper methods
    
    fn cleanup_old_sessions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut sessions = self.list_sessions()?;
        
        if sessions.len() > self.config.max_sessions {
            // Sort by timestamp (oldest first)
            sessions.sort_by(|a, b| a.1.timestamp.cmp(&b.1.timestamp));
            
            // Remove excess sessions
            let excess_count = sessions.len() - self.config.max_sessions;
            for i in 0..excess_count {
                let session_id = &sessions[i].0;
                self.delete_session(session_id)?;
            }
        }
        
        Ok(())
    }
    
    fn capture_session_static(
        browser_state: &BrowserState,
        window_position: Option<(i32, i32)>,
        window_size: Option<(u32, u32)>,
    ) -> SessionData {
        let tabs: Vec<SessionTab> = browser_state
            .tabs
            .values()
            .map(|tab| SessionTab {
                url: tab.url.clone(),
                title: tab.title.clone(),
                scroll_position: None,
                form_data: None,
            })
            .collect();
        
        let active_tab_index = browser_state
            .active_tab_id
            .and_then(|id| {
                browser_state
                    .tabs
                    .keys()
                    .position(|&tab_id| tab_id == id)
            });
        
        SessionData {
            tabs,
            active_tab_index,
            window_position,
            window_size,
            timestamp: chrono::Utc::now(),
            session_name: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::BrowserSettings;
    use tempfile::TempDir;

    #[test]
    fn test_session_capture_and_restore() {
        let temp_dir = TempDir::new().unwrap();
        let session_manager = SessionRestore::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Create test browser state
        let mut browser_state = BrowserState::new();
        browser_state.settings = BrowserSettings::default();
        
        let tab1_id = browser_state.add_tab("https://example.com".to_string());
        let tab2_id = browser_state.add_tab("https://google.com".to_string());
        browser_state.active_tab_id = Some(tab2_id);
        
        // Capture session
        let session = session_manager.capture_session(
            &browser_state,
            Some((100, 100)),
            Some((1280, 800)),
        );
        
        assert_eq!(session.tabs.len(), 2);
        assert_eq!(session.active_tab_index, Some(1)); // Second tab
        assert_eq!(session.window_position, Some((100, 100)));
        
        // Save session
        let session_id = session_manager.save_session(session.clone(), None).unwrap();
        assert!(!session_id.is_empty());
        
        // List sessions
        let sessions = session_manager.list_sessions().unwrap();
        assert!(!sessions.is_empty());
        assert_eq!(sessions[0].0, session_id);
        
        // Restore session
        let restored_session = session_manager.restore_session(&session_id).unwrap();
        assert_eq!(restored_session.tabs.len(), 2);
        assert_eq!(restored_session.active_tab_index, Some(1));
    }

    #[test]
    fn test_session_apply_to_browser() {
        let session_manager = SessionRestore::new(None, None).unwrap();
        
        // Create session data
        let session = SessionData {
            tabs: vec![
                SessionTab {
                    url: "https://example.com".to_string(),
                    title: "Example".to_string(),
                    scroll_position: None,
                    form_data: None,
                },
                SessionTab {
                    url: "https://google.com".to_string(),
                    title: "Google".to_string(),
                    scroll_position: None,
                    form_data: None,
                },
            ],
            active_tab_index: Some(1),
            window_position: Some((100, 100)),
            window_size: Some((1280, 800)),
            timestamp: chrono::Utc::now(),
            session_name: Some("Test Session".to_string()),
        };
        
        // Apply to browser state
        let mut browser_state = BrowserState::new();
        browser_state.settings = BrowserSettings::default();
        
        session_manager
            .apply_session_to_browser(&session, &mut browser_state)
            .unwrap();
        
        assert_eq!(browser_state.tabs.len(), 2);
        assert!(browser_state.active_tab_id.is_some());
        
        let active_tab = browser_state.active_tab().unwrap();
        assert_eq!(active_tab.url, "https://google.com");
    }

    #[test]
    fn test_session_deletion() {
        let temp_dir = TempDir::new().unwrap();
        let session_manager = SessionRestore::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Create and save a session
        let browser_state = BrowserState::new();
        let session = session_manager.capture_session(&browser_state, None, None);
        let session_id = session_manager.save_session(session, Some("Test".to_string())).unwrap();
        
        // Verify session exists
        let sessions_before = session_manager.list_sessions().unwrap();
        assert!(sessions_before.iter().any(|(id, _)| id == &session_id));
        
        // Delete session
        session_manager.delete_session(&session_id).unwrap();
        
        // Verify session is gone
        let sessions_after = session_manager.list_sessions().unwrap();
        assert!(!sessions_after.iter().any(|(id, _)| id == &session_id));
    }
}