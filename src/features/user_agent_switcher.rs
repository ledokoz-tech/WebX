// User Agent Switcher
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Predefined user agents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UserAgentProfile {
    ChromeWindows,
    ChromeMac,
    ChromeLinux,
    FirefoxWindows,
    FirefoxMac,
    FirefoxLinux,
    SafariMac,
    SafariIOS,
    EdgeWindows,
    EdgeMac,
    MobileAndroid,
    MobileIOS,
    Custom(String),
}

/// User agent configuration for a site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SiteUserAgent {
    pub domain_pattern: String,
    pub user_agent: String,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Global user agent settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAgentConfig {
    pub global_user_agent: Option<String>,
    pub per_site_enabled: bool,
    pub randomize_user_agent: bool,
    pub randomization_frequency: RandomizationFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RandomizationFrequency {
    PerRequest,
    PerSession,
    PerHour,
    PerDay,
}

impl Default for UserAgentConfig {
    fn default() -> Self {
        Self {
            global_user_agent: None,
            per_site_enabled: true,
            randomize_user_agent: false,
            randomization_frequency: RandomizationFrequency::PerSession,
        }
    }
}

/// User agent switcher manager
pub struct UserAgentSwitcher {
    config: UserAgentConfig,
    site_agents: Arc<Mutex<HashMap<String, SiteUserAgent>>>,
    current_session_agents: Arc<Mutex<HashMap<String, String>>>,
    config_path: PathBuf,
}

impl UserAgentSwitcher {
    /// Create a new user agent switcher
    pub fn new(
        config: Option<UserAgentConfig>,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("user-agents");
            path
        });
        
        // Create config directory
        fs::create_dir_all(&config_dir)?;
        
        let switcher = Self {
            config,
            site_agents: Arc::new(Mutex::new(HashMap::new())),
            current_session_agents: Arc::new(Mutex::new(HashMap::new())),
            config_path: config_dir.join("config.json"),
        };
        
        // Load existing configuration
        switcher.load_config()?;
        
        Ok(switcher)
    }

    /// Get user agent for a URL
    pub fn get_user_agent(&self, url: &str) -> String {
        // Check for site-specific user agent first
        if self.config.per_site_enabled {
            if let Some(domain) = self.extract_domain(url) {
                let site_agents = self.site_agents.lock().unwrap();
                if let Some(site_agent) = site_agents.get(&domain) {
                    if site_agent.enabled {
                        return site_agent.user_agent.clone();
                    }
                }
            }
        }
        
        // Check for session-specific user agent
        if self.config.randomize_user_agent {
            let mut session_agents = self.current_session_agents.lock().unwrap();
            let domain = self.extract_domain(url).unwrap_or_else(|| "default".to_string());
            
            if !session_agents.contains_key(&domain) {
                let random_ua = self.generate_random_user_agent();
                session_agents.insert(domain.clone(), random_ua);
            }
            
            return session_agents.get(&domain).unwrap().clone();
        }
        
        // Return global user agent or default
        self.config
            .global_user_agent
            .clone()
            .unwrap_or_else(|| self.get_default_user_agent())
    }

    /// Set global user agent
    pub fn set_global_user_agent(&mut self, user_agent: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
        self.config.global_user_agent = user_agent;
        self.save_config()?;
        Ok(())
    }

    /// Set user agent for a specific site
    pub fn set_site_user_agent(
        &self,
        domain_pattern: String,
        user_agent: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let site_agent = SiteUserAgent {
            domain_pattern: domain_pattern.clone(),
            user_agent,
            enabled: true,
            created_at: chrono::Utc::now(),
        };
        
        {
            let mut site_agents = self.site_agents.lock().unwrap();
            site_agents.insert(domain_pattern, site_agent);
        }
        
        self.save_site_agents()?;
        Ok(())
    }

    /// Remove site-specific user agent
    pub fn remove_site_user_agent(&self, domain_pattern: &str) -> bool {
        let mut site_agents = self.site_agents.lock().unwrap();
        let removed = site_agents.remove(domain_pattern).is_some();
        
        if removed {
            let _ = self.save_site_agents();
        }
        
        removed
    }

    /// Enable/disable site-specific user agent
    pub fn set_site_agent_enabled(&self, domain_pattern: &str, enabled: bool) -> bool {
        let mut site_agents = self.site_agents.lock().unwrap();
        if let Some(agent) = site_agents.get_mut(domain_pattern) {
            agent.enabled = enabled;
            let _ = self.save_site_agents();
            true
        } else {
            false
        }
    }

    /// Get all site-specific user agents
    pub fn get_site_agents(&self) -> Vec<SiteUserAgent> {
        let site_agents = self.site_agents.lock().unwrap();
        site_agents.values().cloned().collect()
    }

    /// Set randomization configuration
    pub fn set_randomization(
        &mut self,
        enabled: bool,
        frequency: RandomizationFrequency,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.config.randomize_user_agent = enabled;
        self.config.randomization_frequency = frequency;
        self.save_config()?;
        Ok(())
    }

    /// Clear session user agents
    pub fn clear_session_agents(&self) {
        self.current_session_agents.lock().unwrap().clear();
    }

    /// Reset to default configuration
    pub fn reset_to_defaults(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config = UserAgentConfig::default();
        self.site_agents.lock().unwrap().clear();
        self.current_session_agents.lock().unwrap().clear();
        self.save_config()?;
        self.save_site_agents()?;
        Ok(())
    }

    /// Get available predefined user agent profiles
    pub fn get_predefined_profiles() -> HashMap<UserAgentProfile, String> {
        let mut profiles = HashMap::new();
        
        // Chrome Windows
        profiles.insert(
            UserAgentProfile::ChromeWindows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        );
        
        // Chrome Mac
        profiles.insert(
            UserAgentProfile::ChromeMac,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        );
        
        // Chrome Linux
        profiles.insert(
            UserAgentProfile::ChromeLinux,
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string()
        );
        
        // Firefox Windows
        profiles.insert(
            UserAgentProfile::FirefoxWindows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string()
        );
        
        // Firefox Mac
        profiles.insert(
            UserAgentProfile::FirefoxMac,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:121.0) Gecko/20100101 Firefox/121.0".to_string()
        );
        
        // Firefox Linux
        profiles.insert(
            UserAgentProfile::FirefoxLinux,
            "Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0".to_string()
        );
        
        // Safari Mac
        profiles.insert(
            UserAgentProfile::SafariMac,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15".to_string()
        );
        
        // Safari iOS
        profiles.insert(
            UserAgentProfile::SafariIOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Mobile/15E148 Safari/604.1".to_string()
        );
        
        // Edge Windows
        profiles.insert(
            UserAgentProfile::EdgeWindows,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string()
        );
        
        // Edge Mac
        profiles.insert(
            UserAgentProfile::EdgeMac,
            "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".to_string()
        );
        
        // Mobile Android
        profiles.insert(
            UserAgentProfile::MobileAndroid,
            "Mozilla/5.0 (Linux; Android 14; SM-S918U) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".to_string()
        );
        
        // Mobile iOS
        profiles.insert(
            UserAgentProfile::MobileIOS,
            "Mozilla/5.0 (iPhone; CPU iPhone OS 17_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Mobile/15E148 Safari/604.1".to_string()
        );
        
        profiles
    }

    /// Get JavaScript for user agent spoofing
    pub fn get_user_agent_script(&self) -> String {
        r#"
(function() {
    // Override navigator.userAgent
    const originalUserAgent = navigator.userAgent;
    let currentUserAgent = originalUserAgent;
    
    Object.defineProperty(navigator, 'userAgent', {
        get: function() {
            return currentUserAgent;
        },
        configurable: false
    });
    
    // Override navigator.appVersion
    Object.defineProperty(navigator, 'appVersion', {
        get: function() {
            return currentUserAgent.substring(currentUserAgent.indexOf('/') + 1);
        },
        configurable: false
    });
    
    // Override navigator.platform
    Object.defineProperty(navigator, 'platform', {
        get: function() {
            if (currentUserAgent.includes('Win')) return 'Win32';
            if (currentUserAgent.includes('Mac')) return 'MacIntel';
            if (currentUserAgent.includes('Linux')) return 'Linux x86_64';
            return 'unknown';
        },
        configurable: false
    });
    
    // Listen for user agent updates from the browser
    window.addEventListener('webx-user-agent-change', function(e) {
        currentUserAgent = e.detail.userAgent;
    });
    
    console.log('User agent spoofing enabled');
})();
"#
        .to_string()
    }

    /// Set configuration
    pub fn set_config(&mut self, config: UserAgentConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &UserAgentConfig {
        &self.config
    }

    // Private helper methods
    
    fn extract_domain(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                // Remove www. prefix
                return Some(host.replace("www.", ""));
            }
        }
        None
    }
    
    fn get_default_user_agent(&self) -> String {
        // WebX default user agent
        format!(
            "Mozilla/5.0 ({}; {}) AppleWebKit/537.36 (KHTML, like Gecko) WebX/{} Safari/537.36",
            self.get_platform(),
            self.get_platform_details(),
            env!("CARGO_PKG_VERSION")
        )
    }
    
    fn get_platform(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "Windows NT 10.0; Win64; x64"
        } else if cfg!(target_os = "macos") {
            "Macintosh; Intel Mac OS X 10_15_7"
        } else {
            "X11; Linux x86_64"
        }
    }
    
    fn get_platform_details(&self) -> &'static str {
        if cfg!(target_os = "windows") {
            "Win64"
        } else if cfg!(target_os = "macos") {
            "Mac OS X"
        } else {
            "Linux"
        }
    }
    
    fn generate_random_user_agent(&self) -> String {
        let profiles = Self::get_predefined_profiles();
        let profiles_vec: Vec<&String> = profiles.values().collect();
        
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        
        profiles_vec
            .choose(&mut rng)
            .cloned()
            .unwrap_or(&self.get_default_user_agent())
            .clone()
    }
    
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
    
    fn save_site_agents(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_path.parent().unwrap().join("site_agents.json");
        let site_agents = self.site_agents.lock().unwrap();
        let content = serde_json::to_string_pretty(&*site_agents)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn load_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            self.config = serde_json::from_str(&content)?;
        }
        
        let site_agents_path = self.config_path.parent().unwrap().join("site_agents.json");
        if site_agents_path.exists() {
            let content = fs::read_to_string(&site_agents_path)?;
            let site_agents: HashMap<String, SiteUserAgent> = serde_json::from_str(&content)?;
            *self.site_agents.lock().unwrap() = site_agents;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_user_agent_switching() {
        let temp_dir = TempDir::new().unwrap();
        let mut switcher = UserAgentSwitcher::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test default user agent
        let default_ua = switcher.get_user_agent("https://example.com");
        assert!(!default_ua.is_empty());
        assert!(default_ua.contains("WebX"));
        
        // Test global user agent
        switcher.set_global_user_agent(Some("CustomAgent/1.0".to_string())).unwrap();
        let global_ua = switcher.get_user_agent("https://example.com");
        assert_eq!(global_ua, "CustomAgent/1.0");
    }

    #[test]
    fn test_site_specific_agents() {
        let temp_dir = TempDir::new().unwrap();
        let switcher = UserAgentSwitcher::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Set site-specific user agent
        switcher
            .set_site_user_agent("example.com".to_string(), "SiteSpecificAgent/1.0".to_string())
            .unwrap();
        
        // Test site-specific agent
        let site_ua = switcher.get_user_agent("https://example.com/page");
        assert_eq!(site_ua, "SiteSpecificAgent/1.0");
        
        // Test that other sites still use global/default
        switcher.set_global_user_agent(Some("GlobalAgent/1.0".to_string())).unwrap();
        let other_ua = switcher.get_user_agent("https://google.com");
        assert_eq!(other_ua, "GlobalAgent/1.0");
    }

    #[test]
    fn test_predefined_profiles() {
        let profiles = UserAgentSwitcher::get_predefined_profiles();
        
        // Test that all profiles are present
        assert!(profiles.contains_key(&UserAgentProfile::ChromeWindows));
        assert!(profiles.contains_key(&UserAgentProfile::FirefoxMac));
        assert!(profiles.contains_key(&UserAgentProfile::SafariIOS));
        
        // Test that profiles have reasonable content
        let chrome_ua = profiles.get(&UserAgentProfile::ChromeWindows).unwrap();
        assert!(chrome_ua.contains("Chrome"));
        assert!(chrome_ua.contains("Windows"));
        
        let safari_ua = profiles.get(&UserAgentProfile::SafariIOS).unwrap();
        assert!(safari_ua.contains("Safari"));
        assert!(safari_ua.contains("iPhone"));
    }

    #[test]
    fn test_randomization() {
        let temp_dir = TempDir::new().unwrap();
        let mut switcher = UserAgentSwitcher::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Enable randomization
        switcher
            .set_randomization(true, RandomizationFrequency::PerSession)
            .unwrap();
        
        // Get user agents for same domain - should be consistent
        let ua1 = switcher.get_user_agent("https://example.com/page1");
        let ua2 = switcher.get_user_agent("https://example.com/page2");
        assert_eq!(ua1, ua2);
        
        // Different domain should get different agent
        let ua3 = switcher.get_user_agent("https://google.com");
        assert_ne!(ua1, ua3);
    }

    #[test]
    fn test_configuration_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let mut switcher = UserAgentSwitcher::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Set some configuration
        switcher.set_global_user_agent(Some("PersistentAgent/1.0".to_string())).unwrap();
        switcher
            .set_site_user_agent("test.com".to_string(), "TestAgent/1.0".to_string())
            .unwrap();
        
        // Create new instance to test loading
        let switcher2 = UserAgentSwitcher::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test that configuration was loaded
        let loaded_ua = switcher2.get_user_agent("https://nonexistent.com");
        assert_eq!(loaded_ua, "PersistentAgent/1.0");
        
        let site_agents = switcher2.get_site_agents();
        assert!(!site_agents.is_empty());
        assert_eq!(site_agents[0].domain_pattern, "test.com");
    }
}