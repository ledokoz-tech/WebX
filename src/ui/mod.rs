// WebX Browser UI Module
use crate::core::BrowserState;
use crate::config::ConfigManager;
use crate::features::{TabManager, DownloadManager, PrivacyProtection};
use crate::features::ui::themes::ThemeManager;
use std::sync::{Arc, Mutex};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

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
        let state_arc = Arc::new(Mutex::new(state));
        let tab_manager = Arc::new(TabManager::new(Arc::clone(&state_arc)));
        let download_manager = Arc::new(DownloadManager::new(None)?);
        let privacy_protection = Arc::new(PrivacyProtection::new());
        let theme_manager = Arc::new(ThemeManager::new(None, None)?);
        
        Ok(Self {
            state: state_arc,
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
        let window = BrowserWindow::new(
            &event_loop, 
            self.state.clone(), 
            self.config.clone(),
            self.tab_manager.clone(),
            self.download_manager.clone(),
            self.privacy_protection.clone(),
            self.theme_manager.clone(),
        )?;
        
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
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { device_id: _, event, is_synthetic: _, .. },
                    ..
                } => {
                    // Handle keyboard shortcuts
                    if event.state == tao::event::ElementState::Pressed {
                        // This is where we'd handle Ctrl+T, Ctrl+W, etc.
                        // For now, we'll just log the key press
                        if let tao::keyboard::Key::Character(key_char) = &event.logical_key {
                            println!("Key pressed: {}", key_char);
                        }
                    }
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
