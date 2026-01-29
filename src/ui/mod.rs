// WebX Browser UI Module
use crate::core::{BrowserState, Tab};
use crate::config::ConfigManager;
use crate::features::{TabManager, DownloadManager, PrivacyProtection};
use crate::features::ui::themes::ThemeManager;
use std::sync::{Arc, Mutex};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::WebViewBuilder;

pub mod window;
pub mod menu;

pub use window::BrowserWindow;

/// Main browser application
pub struct BrowserApp {
    state: Arc<Mutex<BrowserState>>,
    config: Arc<ConfigManager>,
    tab_manager: Arc<TabManager>,
    download_manager: Arc<DownloadManager>,
    privacy_protection: Arc<PrivacyProtection>,
    theme_manager: Arc<ThemeManager>,
}

impl BrowserApp {
    /// Create a new browser application
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(ConfigManager::new()?);
        
        let mut state = BrowserState::new();
        
        // Load saved data
        state.settings = config.load_settings();
        state.bookmarks = config.load_bookmarks();
        state.history = config.load_history();
        
        // Initialize feature managers
        let tab_manager = Arc::new(TabManager::new(Arc::clone(&state)));
        let download_manager = Arc::new(DownloadManager::new(None)?);
        let privacy_protection = Arc::new(PrivacyProtection::new(None, None)?);
        let theme_manager = Arc::new(ThemeManager::new(None, None)?);
        
        Ok(Self {
            state: Arc::new(Mutex::new(state)),
            config,
            tab_manager,
            download_manager,
            privacy_protection,
            theme_manager,
        })
    }

    /// Run the browser application
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoop::new();
        
        // Create the main browser window
        let window = BrowserWindow::new(&event_loop, self.state.clone(), self.config.clone())?;
        
        // Run the event loop
        event_loop.run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    // Save state before closing
                    if let Ok(state) = window.state.lock() {
                        let _ = window.config.save_settings(&state.settings);
                        let _ = window.config.save_bookmarks(&state.bookmarks);
                        let _ = window.config.save_history(&state.history);
                    }
                    *control_flow = ControlFlow::Exit;
                }
                _ => {}
            }
        });
    }
}

impl Default for BrowserApp {
    fn default() -> Self {
        Self::new().expect("Failed to create browser app")
    }
}
