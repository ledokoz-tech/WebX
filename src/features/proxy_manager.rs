// Proxy Manager
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Proxy type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProxyType {
    Http,
    Https,
    Socks4,
    Socks5,
}

/// Proxy authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyAuth {
    pub username: String,
    pub password: String,
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub auth: Option<ProxyAuth>,
    pub enabled: bool,
    pub bypass_domains: Vec<String>,
}

/// Proxy profile for different use cases
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProxyProfile {
    None,
    System,
    Custom(String),
    Tor,
    Residential,
    Datacenter,
}

/// Global proxy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalProxySettings {
    pub active_profile: ProxyProfile,
    pub per_domain_profiles: bool,
    pub system_proxy_fallback: bool,
    pub dns_over_proxy: bool,
    pub timeout_seconds: u32,
}

impl Default for GlobalProxySettings {
    fn default() -> Self {
        Self {
            active_profile: ProxyProfile::None,
            per_domain_profiles: false,
            system_proxy_fallback: true,
            dns_over_proxy: false,
            timeout_seconds: 30,
        }
    }
}

/// Proxy manager for handling proxy connections
pub struct ProxyManager {
    settings: GlobalProxySettings,
    profiles: Arc<Mutex<HashMap<ProxyProfile, ProxyConfig>>>,
    domain_profiles: Arc<Mutex<HashMap<String, ProxyProfile>>>,
    config_path: PathBuf,
}

impl ProxyManager {
    /// Create a new proxy manager
    pub fn new(
        settings: Option<GlobalProxySettings>,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let settings = settings.unwrap_or_default();
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("proxies");
            path
        });
        
        // Create config directory
        fs::create_dir_all(&config_dir)?;
        
        let manager = Self {
            settings,
            profiles: Arc::new(Mutex::new(HashMap::new())),
            domain_profiles: Arc::new(Mutex::new(HashMap::new())),
            config_path: config_dir.join("config.json"),
        };
        
        // Load existing configuration
        manager.load_config()?;
        
        // Initialize default profiles
        if manager.profiles.lock().unwrap().is_empty() {
            manager.initialize_default_profiles();
        }
        
        Ok(manager)
    }

    /// Get proxy configuration for a URL
    pub fn get_proxy_for_url(&self, url: &str) -> Option<ProxyConfig> {
        let domain = self.extract_domain(url)?;
        
        // Check for domain-specific profile
        if self.settings.per_domain_profiles {
            let domain_profiles = self.domain_profiles.lock().unwrap();
            if let Some(profile) = domain_profiles.get(&domain) {
                if let Some(config) = self.get_profile_config(profile) {
                    if config.enabled {
                        return Some(config);
                    }
                }
            }
        }
        
        // Use active profile
        if let Some(config) = self.get_profile_config(&self.settings.active_profile) {
            if config.enabled {
                return Some(config);
            }
        }
        
        // Fallback to system proxy if enabled
        if self.settings.system_proxy_fallback {
            if let Some(system_proxy) = self.get_system_proxy() {
                return Some(system_proxy);
            }
        }
        
        None
    }

    /// Set active proxy profile
    pub fn set_active_profile(&mut self, profile: ProxyProfile) -> Result<(), Box<dyn std::error::Error>> {
        self.settings.active_profile = profile;
        self.save_config()?;
        Ok(())
    }

    /// Set proxy configuration for a specific domain
    pub fn set_domain_profile(&self, domain: String, profile: ProxyProfile) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut domain_profiles = self.domain_profiles.lock().unwrap();
            domain_profiles.insert(domain, profile);
        }
        self.save_domain_profiles()?;
        Ok(())
    }

    /// Remove domain-specific proxy profile
    pub fn remove_domain_profile(&self, domain: &str) -> bool {
        let mut domain_profiles = self.domain_profiles.lock().unwrap();
        let removed = domain_profiles.remove(domain).is_some();
        
        if removed {
            let _ = self.save_domain_profiles();
        }
        
        removed
    }

    /// Add custom proxy profile
    pub fn add_custom_proxy(
        &self,
        name: String,
        config: ProxyConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let profile = ProxyProfile::Custom(name);
        
        {
            let mut profiles = self.profiles.lock().unwrap();
            profiles.insert(profile, config);
        }
        
        self.save_profiles()?;
        Ok(())
    }

    /// Remove custom proxy profile
    pub fn remove_custom_proxy(&self, name: &str) -> bool {
        let mut profiles = self.profiles.lock().unwrap();
        let removed = profiles.remove(&ProxyProfile::Custom(name.to_string())).is_some();
        
        if removed {
            // Also remove any domain assignments to this profile
            let mut domain_profiles = self.domain_profiles.lock().unwrap();
            domain_profiles.retain(|_, profile| {
                if let ProxyProfile::Custom(profile_name) = profile {
                    profile_name != name
                } else {
                    true
                }
            });
            
            let _ = self.save_profiles();
            let _ = self.save_domain_profiles();
        }
        
        removed
    }

    /// Get all available proxy profiles
    pub fn get_available_profiles(&self) -> Vec<ProxyProfile> {
        let profiles = self.profiles.lock().unwrap();
        profiles.keys().cloned().collect()
    }

    /// Get configuration for a specific profile
    pub fn get_profile_config(&self, profile: &ProxyProfile) -> Option<ProxyConfig> {
        let profiles = self.profiles.lock().unwrap();
        profiles.get(profile).cloned()
    }

    /// Test proxy connectivity
    pub async fn test_proxy_connectivity(&self, config: &ProxyConfig) -> Result<bool, Box<dyn std::error::Error>> {
        // This would test actual connectivity to the proxy
        // For demo purposes, we'll simulate a test
        use tokio::time::{timeout, Duration};
        
        let result = timeout(Duration::from_secs(self.settings.timeout_seconds as u64), async {
            // Simulate connection test
            tokio::time::sleep(Duration::from_millis(100)).await;
            true // Simulate success
        }).await;
        
        Ok(result.unwrap_or(false))
    }

    /// Get proxy PAC (Proxy Auto-Configuration) script
    pub fn get_pac_script(&self) -> String {
        let profiles = self.profiles.lock().unwrap();
        let domain_profiles = self.domain_profiles.lock().unwrap();
        
        let mut pac_rules = String::new();
        
        // Domain-specific rules
        for (domain, profile) in domain_profiles.iter() {
            if let Some(config) = profiles.get(profile) {
                if config.enabled {
                    let proxy_string = self.proxy_config_to_pac(config);
                    pac_rules.push_str(&format!(
                        "  if (shExpMatch(host, '{}')) return '{}';\n",
                        domain, proxy_string
                    ));
                }
            }
        }
        
        // Active profile rule
        if let Some(config) = profiles.get(&self.settings.active_profile) {
            if config.enabled {
                let proxy_string = self.proxy_config_to_pac(config);
                pac_rules.push_str(&format!("  return '{}';\n", proxy_string));
            } else {
                pac_rules.push_str("  return 'DIRECT';\n");
            }
        } else {
            pac_rules.push_str("  return 'DIRECT';\n");
        }
        
        format!(
            r#"function FindProxyForURL(url, host) {{
  // Domain-specific proxy rules
{}
  
  // Default: direct connection
  return 'DIRECT';
}}"#,
            pac_rules
        )
    }

    /// Set global settings
    pub fn set_settings(&mut self, settings: GlobalProxySettings) {
        self.settings = settings;
    }

    /// Get current settings
    pub fn get_settings(&self) -> &GlobalProxySettings {
        &self.settings
    }

    /// Get system proxy configuration (platform-specific)
    pub fn get_system_proxy(&self) -> Option<ProxyConfig> {
        // This would query system proxy settings
        // Implementation varies by platform
        #[cfg(target_os = "windows")]
        {
            self.get_windows_system_proxy()
        }
        #[cfg(target_os = "macos")]
        {
            self.get_macos_system_proxy()
        }
        #[cfg(target_os = "linux")]
        {
            self.get_linux_system_proxy()
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            None
        }
    }

    /// Clear all proxy settings
    pub fn clear_all_settings(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.settings = GlobalProxySettings::default();
        self.profiles.lock().unwrap().clear();
        self.domain_profiles.lock().unwrap().clear();
        self.initialize_default_profiles();
        self.save_config()?;
        self.save_profiles()?;
        self.save_domain_profiles()?;
        Ok(())
    }

    // Private helper methods
    
    fn extract_domain(&self, url: &str) -> Option<String> {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                return Some(host.replace("www.", ""));
            }
        }
        None
    }
    
    fn initialize_default_profiles(&self) {
        let mut profiles = self.profiles.lock().unwrap();
        
        // No proxy
        profiles.insert(
            ProxyProfile::None,
            ProxyConfig {
                proxy_type: ProxyType::Http,
                host: String::new(),
                port: 0,
                auth: None,
                enabled: false,
                bypass_domains: vec![],
            },
        );
        
        // System proxy
        profiles.insert(
            ProxyProfile::System,
            ProxyConfig {
                proxy_type: ProxyType::Http,
                host: "system".to_string(),
                port: 0,
                auth: None,
                enabled: false,
                bypass_domains: vec![],
            },
        );
        
        // Tor proxy (localhost:9050)
        profiles.insert(
            ProxyProfile::Tor,
            ProxyConfig {
                proxy_type: ProxyType::Socks5,
                host: "127.0.0.1".to_string(),
                port: 9050,
                auth: None,
                enabled: false,
                bypass_domains: vec![],
            },
        );
    }
    
    fn proxy_config_to_pac(&self, config: &ProxyConfig) -> String {
        let proxy_type = match config.proxy_type {
            ProxyType::Http | ProxyType::Https => "PROXY",
            ProxyType::Socks4 => "SOCKS4",
            ProxyType::Socks5 => "SOCKS5",
        };
        
        format!("{} {}:{}", proxy_type, config.host, config.port)
    }
    
    #[cfg(target_os = "windows")]
    fn get_windows_system_proxy(&self) -> Option<ProxyConfig> {
        // Windows system proxy detection
        // This would use Windows API calls
        None // Placeholder
    }
    
    #[cfg(target_os = "macos")]
    fn get_macos_system_proxy(&self) -> Option<ProxyConfig> {
        // macOS system proxy detection
        // This would use System Configuration framework
        None // Placeholder
    }
    
    #[cfg(target_os = "linux")]
    fn get_linux_system_proxy(&self) -> Option<ProxyConfig> {
        // Linux system proxy detection
        // This would check environment variables and desktop settings
        if let Ok(http_proxy) = std::env::var("http_proxy") {
            if let Ok(url) = url::Url::parse(&http_proxy) {
                if let Some(host) = url.host_str() {
                    return Some(ProxyConfig {
                        proxy_type: ProxyType::Http,
                        host: host.to_string(),
                        port: url.port().unwrap_or(8080),
                        auth: None,
                        enabled: true,
                        bypass_domains: vec![],
                    });
                }
            }
        }
        None
    }
    
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(&self.settings)?;
        fs::write(&self.config_path, content)?;
        Ok(())
    }
    
    fn save_profiles(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_path.parent().unwrap().join("profiles.json");
        let profiles = self.profiles.lock().unwrap();
        let content = serde_json::to_string_pretty(&*profiles)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn save_domain_profiles(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_path.parent().unwrap().join("domain_profiles.json");
        let domain_profiles = self.domain_profiles.lock().unwrap();
        let content = serde_json::to_string_pretty(&*domain_profiles)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn load_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            let content = fs::read_to_string(&self.config_path)?;
            self.settings = serde_json::from_str(&content)?;
        }
        
        let profiles_path = self.config_path.parent().unwrap().join("profiles.json");
        if profiles_path.exists() {
            let content = fs::read_to_string(&profiles_path)?;
            let profiles: HashMap<ProxyProfile, ProxyConfig> = serde_json::from_str(&content)?;
            *self.profiles.lock().unwrap() = profiles;
        }
        
        let domain_profiles_path = self.config_path.parent().unwrap().join("domain_profiles.json");
        if domain_profiles_path.exists() {
            let content = fs::read_to_string(&domain_profiles_path)?;
            let domain_profiles: HashMap<String, ProxyProfile> = serde_json::from_str(&content)?;
            *self.domain_profiles.lock().unwrap() = domain_profiles;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_proxy_profile_management() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProxyManager::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test default profiles exist
        let profiles = manager.get_available_profiles();
        assert!(profiles.contains(&ProxyProfile::None));
        assert!(profiles.contains(&ProxyProfile::System));
        assert!(profiles.contains(&ProxyProfile::Tor));
        
        // Test getting profile config
        let none_config = manager.get_profile_config(&ProxyProfile::None);
        assert!(none_config.is_some());
        assert!(!none_config.unwrap().enabled);
    }

    #[test]
    fn test_domain_specific_proxies() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProxyManager::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Enable Tor profile for testing
        if let Some(mut tor_config) = manager.get_profile_config(&ProxyProfile::Tor) {
            tor_config.enabled = true;
            
            let mut profiles = manager.profiles.lock().unwrap();
            profiles.insert(ProxyProfile::Tor, tor_config);
        }
        
        // Set domain-specific profile
        manager
            .set_domain_profile("example.com".to_string(), ProxyProfile::Tor)
            .unwrap();
        
        // Test proxy resolution
        let proxy_config = manager.get_proxy_for_url("https://example.com/page");
        assert!(proxy_config.is_some());
        assert_eq!(proxy_config.unwrap().port, 9050); // Tor port
        
        // Test that other domains use default
        let other_proxy = manager.get_proxy_for_url("https://google.com");
        assert!(other_proxy.is_none()); // Should be None since no active profile
    }

    #[test]
    fn test_custom_proxy_profiles() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ProxyManager::new(None, Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Add custom proxy
        let custom_config = ProxyConfig {
            proxy_type: ProxyType::Http,
            host: "proxy.example.com".to_string(),
            port: 8080,
            auth: Some(ProxyAuth {
                username: "user".to_string(),
                password: "pass".to_string(),
            }),
            enabled: true,
            bypass_domains: vec!["internal.com".to_string()],
        };
        
        manager
            .add_custom_proxy("MyProxy".to_string(), custom_config)
            .unwrap();
        
        // Test custom profile exists
        let profiles = manager.get_available_profiles();
        assert!(profiles.iter().any(|p| {
            if let ProxyProfile::Custom(name) = p {
                name == "MyProxy"
            } else {
                false
            }
        }));
        
        // Test removing custom proxy
        assert!(manager.remove_custom_proxy("MyProxy"));
        let profiles_after = manager.get_available_profiles();
        assert!(!profiles_after.iter().any(|p| {
            if let ProxyProfile::Custom(name) = p {
                name == "MyProxy"
            } else {
                false
            }
        }));
    }

    #[test]
    fn test_pac_script_generation() {
        let manager = ProxyManager::new(None, None).unwrap();
        
        // Add a domain profile for testing
        if let Some(mut tor_config) = manager.get_profile_config(&ProxyProfile::Tor) {
            tor_config.enabled = true;
            
            let mut profiles = manager.profiles.lock().unwrap();
            profiles.insert(ProxyProfile::Tor, tor_config);
        }
        
        manager
            .set_domain_profile("example.com".to_string(), ProxyProfile::Tor)
            .unwrap();
        
        let pac_script = manager.get_pac_script();
        assert!(pac_script.contains("FindProxyForURL"));
        assert!(pac_script.contains("example.com"));
        assert!(pac_script.contains("SOCKS5 127.0.0.1:9050"));
    }

    #[tokio::test]
    async fn test_proxy_connectivity() {
        let manager = ProxyManager::new(None, None).unwrap();
        
        // Test with a dummy configuration
        let config = ProxyConfig {
            proxy_type: ProxyType::Http,
            host: "127.0.0.1".to_string(),
            port: 8080,
            auth: None,
            enabled: true,
            bypass_domains: vec![],
        };
        
        // This should not panic and return a result
        let result = manager.test_proxy_connectivity(&config).await;
        assert!(result.is_ok());
    }
}