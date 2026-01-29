// Browser window implementation
use crate::core::BrowserState;
use crate::config::ConfigManager;
use crate::features::{TabManager, DownloadManager, PrivacyProtection};
use crate::features::ui::themes::ThemeManager;
use std::sync::{Arc, Mutex};
use tao::{
    dpi::{LogicalSize, PhysicalPosition},
    event_loop::EventLoop,
    window::{Window, WindowBuilder, Icon},
};
use wry::{WebView, WebViewBuilder};

/// Main browser window
pub struct BrowserWindow {
    pub window: Window,
    pub webview: WebView,
    pub state: Arc<Mutex<BrowserState>>,
    pub config: Arc<ConfigManager>,
    pub tab_manager: Arc<TabManager>,
    pub download_manager: Arc<DownloadManager>,
    pub privacy_protection: Arc<PrivacyProtection>,
    pub theme_manager: Arc<ThemeManager>,
}

impl BrowserWindow {
    /// Create a new browser window
    pub fn new(
        event_loop: &EventLoop<()>,
        state: Arc<Mutex<BrowserState>>,
        config: Arc<ConfigManager>,
        tab_manager: Arc<TabManager>,
        download_manager: Arc<DownloadManager>,
        privacy_protection: Arc<PrivacyProtection>,
        theme_manager: Arc<ThemeManager>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        
        // Create the window
        let window = WindowBuilder::new()
            .with_title("WebX Browser - Ledokoz OS")
            .with_inner_size(LogicalSize::new(1280, 800))
            .with_min_inner_size(LogicalSize::new(800, 600))
            .build(event_loop)?;

        // Get the initial URL
        let initial_url = {
            let mut state_lock = state.lock().unwrap();
            let url = state_lock.settings.home_page.clone();
            state_lock.add_tab(url.clone());
            url
        };

        // Build the webview with custom HTML UI
        let webview = WebViewBuilder::new(&window)
            .with_url(&initial_url)
            .with_devtools(true)
            .with_initialization_script(include_str!("scripts/init.js"))
            .with_ipc_handler(move |request| {
                // Handle IPC messages from the webview
                tracing::info!("IPC message: {}", request.body());
            })
            .build()?;

        Ok(Self {
            window,
            webview,
            state,
            config,
            tab_manager,
            download_manager,
            privacy_protection,
            theme_manager,
        })
    }

    /// Navigate to a URL
    pub fn navigate(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.webview.load_url(url)?;
        
        // Update state
        if let Ok(mut state) = self.state.lock() {
            if let Some(tab) = state.active_tab_mut() {
                tab.url = url.to_string();
                tab.is_loading = true;
            }
        }
        
        Ok(())
    }

    /// Go back in history
    pub fn go_back(&self) -> Result<(), Box<dyn std::error::Error>> {
        // WebView doesn't expose history navigation directly
        // We'll need to implement our own history management
        Ok(())
    }

    /// Go forward in history
    pub fn go_forward(&self) -> Result<(), Box<dyn std::error::Error>> {
        // WebView doesn't expose history navigation directly
        // We'll need to implement our own history management
        Ok(())
    }

    /// Reload the current page
    pub fn reload(&self) -> Result<(), Box<dyn std::error::Error>> {
        if let Ok(state) = self.state.lock() {
            if let Some(tab) = state.active_tab() {
                self.webview.load_url(&tab.url)?;
            }
        }
        Ok(())
    }

    /// Execute JavaScript in the webview
    pub fn eval_script(&self, script: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.webview.evaluate_script(script)?;
        Ok(())
    }

    /// Set the window title
    pub fn set_title(&self, title: &str) {
        self.window.set_title(title);
    }

    /// Get the window title
    pub fn title(&self) -> String {
        if let Ok(state) = self.state.lock() {
            if let Some(tab) = state.active_tab() {
                return format!("{} - WebX Browser", tab.title);
            }
        }
        "WebX Browser".to_string()
    }
}
