// Privacy-Focused Tracking Protection
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Tracking protection level
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProtectionLevel {
    Minimal,    // Block only known trackers
    Balanced,   // Block most trackers while maintaining usability
    Strict,     // Maximum protection, may break some sites
    Custom,     // User-defined rules
}

/// Tracker category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TrackerCategory {
    Advertising,
    Analytics,
    SocialMedia,
    Cryptomining,
    Fingerprinting,
    EmailTracking,
    Affiliate,
    CDN,
}

/// Tracking protection rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingRule {
    pub pattern: String,
    pub category: TrackerCategory,
    pub is_regex: bool,
    pub enabled: bool,
    pub description: String,
    pub added_at: chrono::DateTime<chrono::Utc>,
}

/// Privacy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    pub protection_level: ProtectionLevel,
    pub block_third_party_cookies: bool,
    pub block_fingerprinting: bool,
    pub clear_data_on_exit: bool,
    pub do_not_track: bool,
    pub strict_https: bool,
    pub disable_referrer: bool,
    pub custom_rules_enabled: bool,
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            protection_level: ProtectionLevel::Balanced,
            block_third_party_cookies: true,
            block_fingerprinting: true,
            clear_data_on_exit: false,
            do_not_track: true,
            strict_https: true,
            disable_referrer: false,
            custom_rules_enabled: true,
        }
    }
}

/// Tracking protection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyStats {
    pub trackers_blocked: u64,
    pub cookies_blocked: u64,
    pub fingerprinting_attempts: u64,
    pub https_upgrades: u64,
    pub start_time: chrono::DateTime<chrono::Utc>,
}

impl Default for PrivacyStats {
    fn default() -> Self {
        Self {
            trackers_blocked: 0,
            cookies_blocked: 0,
            fingerprinting_attempts: 0,
            https_upgrades: 0,
            start_time: chrono::Utc::now(),
        }
    }
}

/// Privacy-focused tracking protection manager
pub struct PrivacyProtection {
    config: PrivacyConfig,
    rules: Arc<Mutex<Vec<TrackingRule>>>,
    compiled_patterns: Arc<Mutex<HashMap<TrackerCategory, Vec<Regex>>>>,
    stats: Arc<Mutex<PrivacyStats>>,
    config_dir: PathBuf,
}

impl PrivacyProtection {
    /// Create a new privacy protection manager
    pub fn new(
        config: Option<PrivacyConfig>,
        config_dir: Option<PathBuf>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("privacy");
            path
        });
        
        // Create config directory
        fs::create_dir_all(&config_dir)?;
        
        let protection = Self {
            config,
            rules: Arc::new(Mutex::new(Vec::new())),
            compiled_patterns: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(PrivacyStats::default())),
            config_dir,
        };
        
        // Load rules based on protection level
        protection.load_rules_for_level()?;
        protection.compile_patterns()?;
        
        Ok(protection)
    }

    /// Check if a URL should be blocked
    pub fn should_block_url(&self, url: &str, category: &TrackerCategory) -> bool {
        if !self.is_category_enabled(category) {
            return false;
        }
        
        let patterns = self.compiled_patterns.lock().unwrap();
        if let Some(regexes) = patterns.get(category) {
            for regex in regexes {
                if regex.is_match(url) {
                    self.increment_blocked_tracker();
                    return true;
                }
            }
        }
        
        false
    }

    /// Check if a cookie should be blocked
    pub fn should_block_cookie(&self, domain: &str, is_third_party: bool) -> bool {
        if !self.config.block_third_party_cookies {
            return false;
        }
        
        if is_third_party {
            self.increment_blocked_cookie();
            true
        } else {
            false
        }
    }

    /// Detect fingerprinting attempts
    pub fn detect_fingerprinting(&self, script_content: &str) -> bool {
        if !self.config.block_fingerprinting {
            return false;
        }
        
        // Common fingerprinting techniques
        let fingerprinting_patterns = [
            "canvas.fingerprint",
            "webgl.fingerprint",
            "audiocontext.fingerprint",
            "navigator.deviceMemory",
            "navigator.hardwareConcurrency",
            "screen.colorDepth",
            "performance.memory",
        ];
        
        for pattern in &fingerprinting_patterns {
            if script_content.contains(pattern) {
                self.increment_fingerprinting_attempt();
                return true;
            }
        }
        
        false
    }

    /// Upgrade HTTP URL to HTTPS if possible
    pub fn upgrade_to_https(&self, url: &str) -> Option<String> {
        if !self.config.strict_https {
            return None;
        }
        
        if url.starts_with("http://") {
            let https_url = url.replacen("http://", "https://", 1);
            self.increment_https_upgrade();
            Some(https_url)
        } else {
            None
        }
    }

    /// Modify referrer header
    pub fn modify_referrer(&self, referrer: Option<&str>, destination: &str) -> Option<String> {
        if !self.config.disable_referrer {
            return referrer.map(|s| s.to_string());
        }
        
        // Strip referrer for cross-origin requests
        if let Some(referrer_url) = referrer {
            if let (Ok(ref_src), Ok(ref_dest)) = (
                url::Url::parse(referrer_url),
                url::Url::parse(destination),
            ) {
                if ref_src.origin() != ref_dest.origin() {
                    return None; // No referrer for cross-origin
                }
            }
        }
        
        referrer.map(|s| s.to_string())
    }

    /// Get privacy headers to inject
    pub fn get_privacy_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        
        if self.config.do_not_track {
            headers.insert("DNT".to_string(), "1".to_string());
        }
        
        headers.insert(
            "Sec-GPC".to_string(), // Global Privacy Control
            "1".to_string(),
        );
        
        headers
    }

    /// Add custom tracking rule
    pub fn add_custom_rule(
        &self,
        pattern: String,
        category: TrackerCategory,
        description: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let rule = TrackingRule {
            pattern: pattern.clone(),
            category,
            is_regex: true,
            enabled: true,
            description,
            added_at: chrono::Utc::now(),
        };
        
        {
            let mut rules = self.rules.lock().unwrap();
            rules.push(rule);
        }
        
        self.compile_patterns()?;
        self.save_custom_rules()?;
        
        Ok(())
    }

    /// Remove custom tracking rule
    pub fn remove_custom_rule(&self, pattern: &str) -> bool {
        let mut rules = self.rules.lock().unwrap();
        let initial_len = rules.len();
        rules.retain(|rule| rule.pattern != pattern);
        
        if rules.len() != initial_len {
            let _ = self.compile_patterns();
            let _ = self.save_custom_rules();
            true
        } else {
            false
        }
    }

    /// Set protection level
    pub fn set_protection_level(&mut self, level: ProtectionLevel) -> Result<(), Box<dyn std::error::Error>> {
        self.config.protection_level = level;
        self.load_rules_for_level()?;
        self.compile_patterns()?;
        self.save_config()?;
        Ok(())
    }

    /// Get current statistics
    pub fn get_statistics(&self) -> PrivacyStats {
        self.stats.lock().unwrap().clone()
    }

    /// Reset statistics
    pub fn reset_statistics(&self) {
        *self.stats.lock().unwrap() = PrivacyStats::default();
    }

    /// Clear browsing data
    pub fn clear_browsing_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        // This would clear cookies, cache, history, etc.
        // Implementation depends on the storage backend
        tracing::info!("Clearing browsing data...");
        Ok(())
    }

    /// Get JavaScript for anti-fingerprinting
    pub fn get_anti_fingerprinting_script(&self) -> String {
        if !self.config.block_fingerprinting {
            return String::new();
        }
        
        r#"
(function() {
    // Canvas fingerprinting protection
    const canvas = document.createElement('canvas');
    const originalGetContext = canvas.getContext;
    
    canvas.getContext = function() {
        const context = originalGetContext.apply(this, arguments);
        if (context) {
            // Add noise to canvas operations
            const originalFillText = context.fillText;
            context.fillText = function() {
                // Add slight randomness
                arguments[1] += (Math.random() - 0.5) * 0.0001;
                arguments[2] += (Math.random() - 0.5) * 0.0001;
                return originalFillText.apply(this, arguments);
            };
        }
        return context;
    };
    
    // WebGL fingerprinting protection
    const originalGetParameter = WebGLRenderingContext.prototype.getParameter;
    WebGLRenderingContext.prototype.getParameter = function(parameter) {
        // Return slightly randomized values for fingerprintable parameters
        const value = originalGetParameter.call(this, parameter);
        if (parameter === this.VERSION || parameter === this.SHADING_LANGUAGE_VERSION) {
            return value + Math.random().toString(36).substr(2, 5);
        }
        return value;
    };
    
    // Audio fingerprinting protection
    const originalCreateAnalyser = AudioContext.prototype.createAnalyser;
    AudioContext.prototype.createAnalyser = function() {
        const analyser = originalCreateAnalyser.call(this);
        // Add noise to audio analysis
        const originalGetByteFrequencyData = analyser.getByteFrequencyData;
        analyser.getByteFrequencyData = function(array) {
            originalGetByteFrequencyData.call(this, array);
            // Add random noise
            for (let i = 0; i < array.length; i++) {
                array[i] = Math.min(255, Math.max(0, array[i] + (Math.random() - 0.5) * 2));
            }
        };
        return analyser;
    };
    
    console.log('Anti-fingerprinting protections enabled');
})();
"#
        .to_string()
    }

    /// Set configuration
    pub fn set_config(&mut self, config: PrivacyConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &PrivacyConfig {
        &self.config
    }

    // Private helper methods
    
    fn is_category_enabled(&self, category: &TrackerCategory) -> bool {
        match self.config.protection_level {
            ProtectionLevel::Minimal => matches!(
                category,
                TrackerCategory::Advertising | TrackerCategory::Analytics
            ),
            ProtectionLevel::Balanced => !matches!(category, TrackerCategory::CDN),
            ProtectionLevel::Strict => true,
            ProtectionLevel::Custom => {
                // Check if custom rules exist for this category
                let rules = self.rules.lock().unwrap();
                rules.iter().any(|rule| rule.category == *category && rule.enabled)
            }
        }
    }
    
    fn load_rules_for_level(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut rules = self.rules.lock().unwrap();
        rules.clear();
        
        // Load base rules depending on protection level
        let base_rules = match self.config.protection_level {
            ProtectionLevel::Minimal => self.get_minimal_rules(),
            ProtectionLevel::Balanced => self.get_balanced_rules(),
            ProtectionLevel::Strict => self.get_strict_rules(),
            ProtectionLevel::Custom => vec![], // Will load custom rules from file
        };
        
        rules.extend(base_rules);
        
        // Load custom rules if enabled
        if self.config.custom_rules_enabled {
            self.load_custom_rules()?;
        }
        
        Ok(())
    }
    
    fn compile_patterns(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules = self.rules.lock().unwrap();
        let mut patterns: HashMap<TrackerCategory, Vec<Regex>> = HashMap::new();
        
        for rule in rules.iter().filter(|r| r.enabled) {
            let regex = if rule.is_regex {
                Regex::new(&rule.pattern)?
            } else {
                Regex::new(&regex::escape(&rule.pattern))?
            };
            
            patterns
                .entry(rule.category.clone())
                .or_insert_with(Vec::new)
                .push(regex);
        }
        
        *self.compiled_patterns.lock().unwrap() = patterns;
        Ok(())
    }
    
    fn increment_blocked_tracker(&self) {
        self.stats.lock().unwrap().trackers_blocked += 1;
    }
    
    fn increment_blocked_cookie(&self) {
        self.stats.lock().unwrap().cookies_blocked += 1;
    }
    
    fn increment_fingerprinting_attempt(&self) {
        self.stats.lock().unwrap().fingerprinting_attempts += 1;
    }
    
    fn increment_https_upgrade(&self) {
        self.stats.lock().unwrap().https_upgrades += 1;
    }
    
    fn get_minimal_rules(&self) -> Vec<TrackingRule> {
        vec![
            self.create_rule(".*\\.doubleclick\\.net.*", TrackerCategory::Advertising, "Google DoubleClick"),
            self.create_rule(".*\\.googlesyndication\\.com.*", TrackerCategory::Advertising, "Google Ads"),
            self.create_rule(".*\\.google-analytics\\.com.*", TrackerCategory::Analytics, "Google Analytics"),
        ]
    }
    
    fn get_balanced_rules(&self) -> Vec<TrackingRule> {
        let mut rules = self.get_minimal_rules();
        rules.extend(vec![
            self.create_rule(".*\\.facebook\\.com.*", TrackerCategory::SocialMedia, "Facebook Tracker"),
            self.create_rule(".*\\.twitter\\.com.*", TrackerCategory::SocialMedia, "Twitter Tracker"),
            self.create_rule(".*\\.linkedin\\.com.*", TrackerCategory::SocialMedia, "LinkedIn Tracker"),
            self.create_rule(".*\\.adservice\\.google\\.com.*", TrackerCategory::Advertising, "Google AdService"),
        ]);
        rules
    }
    
    fn get_strict_rules(&self) -> Vec<TrackingRule> {
        let mut rules = self.get_balanced_rules();
        rules.extend(vec![
            self.create_rule(".*\\.cloudflare\\.com.*", TrackerCategory::CDN, "Cloudflare Tracking"),
            self.create_rule(".*\\.akamai\\.com.*", TrackerCategory::CDN, "Akamai Tracking"),
        ]);
        rules
    }
    
    fn create_rule(&self, pattern: &str, category: TrackerCategory, description: &str) -> TrackingRule {
        TrackingRule {
            pattern: pattern.to_string(),
            category,
            is_regex: true,
            enabled: true,
            description: description.to_string(),
            added_at: chrono::Utc::now(),
        }
    }
    
    fn load_custom_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("custom_rules.json");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let custom_rules: Vec<TrackingRule> = serde_json::from_str(&content)?;
            
            let mut rules = self.rules.lock().unwrap();
            rules.extend(custom_rules);
        }
        Ok(())
    }
    
    fn save_custom_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("custom_rules.json");
        let rules = self.rules.lock().unwrap();
        let custom_rules: Vec<&TrackingRule> = rules.iter()
            .filter(|rule| rule.added_at.timestamp() > 0) // Filter for custom-added rules
            .collect();
        
        let content = serde_json::to_string_pretty(&custom_rules)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("config.json");
        let content = serde_json::to_string_pretty(&self.config)?;
        fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tracking_protection_levels() {
        let temp_dir = TempDir::new().unwrap();
        
        // Test minimal protection
        let mut config = PrivacyConfig::default();
        config.protection_level = ProtectionLevel::Minimal;
        let protection = PrivacyProtection::new(Some(config), Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Should block advertising trackers
        assert!(protection.should_block_url("https://ads.doubleclick.net/ad", &TrackerCategory::Advertising));
        // Should not block CDN trackers in minimal mode
        assert!(!protection.should_block_url("https://cdn.cloudflare.com/script.js", &TrackerCategory::CDN));
        
        // Test strict protection
        let mut config = PrivacyConfig::default();
        config.protection_level = ProtectionLevel::Strict;
        let protection = PrivacyProtection::new(Some(config), Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Should block everything
        assert!(protection.should_block_url("https://cdn.cloudflare.com/script.js", &TrackerCategory::CDN));
    }

    #[test]
    fn test_cookie_blocking() {
        let protection = PrivacyProtection::new(None, None).unwrap();
        
        // Enable third-party cookie blocking
        let mut config = protection.get_config().clone();
        config.block_third_party_cookies = true;
        // Note: We can't directly modify config in this test setup
        
        // Test third-party cookie blocking
        assert!(protection.should_block_cookie("thirdparty.com", true));
        // Should not block first-party cookies
        assert!(!protection.should_block_cookie("example.com", false));
    }

    #[test]
    fn test_fingerprinting_detection() {
        let protection = PrivacyProtection::new(None, None).unwrap();
        
        // Test fingerprinting detection
        let fingerprinting_script = "var canvas = document.createElement('canvas'); canvas.fingerprint();";
        assert!(protection.detect_fingerprinting(fingerprinting_script));
        
        let normal_script = "console.log('normal script');";
        assert!(!protection.detect_fingerprinting(normal_script));
    }

    #[test]
    fn test_https_upgrade() {
        let protection = PrivacyProtection::new(None, None).unwrap();
        
        // Enable strict HTTPS
        let mut config = protection.get_config().clone();
        config.strict_https = true;
        // Note: Direct config modification not possible in this test setup
        
        // Test HTTP to HTTPS upgrade
        let upgraded = protection.upgrade_to_https("http://example.com");
        assert_eq!(upgraded, Some("https://example.com".to_string()));
        
        // Should not upgrade already HTTPS URLs
        let upgraded = protection.upgrade_to_https("https://example.com");
        assert_eq!(upgraded, None);
    }

    #[test]
    fn test_statistics() {
        let protection = PrivacyProtection::new(None, None).unwrap();
        
        // Test statistics tracking
        let initial_stats = protection.get_statistics();
        assert_eq!(initial_stats.trackers_blocked, 0);
        
        // Simulate blocking (would normally happen internally)
        // In a real test, we'd trigger the blocking logic
    }
}