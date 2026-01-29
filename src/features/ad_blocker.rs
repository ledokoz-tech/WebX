// Ad Blocker with Customizable Filter Lists
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Ad blocker rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AdBlockRule {
    pub pattern: String,
    pub is_regex: bool,
    pub enabled: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Predefined filter lists
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterList {
    EasyList,
    EasyPrivacy,
    FanboyAnnoyances,
    Custom,
}

/// Ad blocker for blocking advertisements and trackers
pub struct AdBlocker {
    rules: Arc<Mutex<HashSet<AdBlockRule>>>,
    compiled_regexes: Arc<Mutex<Vec<Regex>>>,
    config_dir: PathBuf,
}

impl AdBlocker {
    /// Create a new ad blocker
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("adblock");
            path
        });
        
        // Create config directory
        fs::create_dir_all(&config_dir)?;
        
        let blocker = Self {
            rules: Arc::new(Mutex::new(HashSet::new())),
            compiled_regexes: Arc::new(Mutex::new(Vec::new())),
            config_dir,
        };
        
        // Load existing rules
        blocker.load_rules()?;
        
        // Add default rules if none exist
        if blocker.rules.lock().unwrap().is_empty() {
            blocker.add_default_rules()?;
        }
        
        Ok(blocker)
    }

    /// Check if a URL should be blocked
    pub fn should_block(&self, url: &str) -> bool {
        let regexes = self.compiled_regexes.lock().unwrap();
        
        for regex in regexes.iter() {
            if regex.is_match(url) {
                return true;
            }
        }
        
        false
    }

    /// Add a new blocking rule
    pub fn add_rule(&self, pattern: String, is_regex: bool) -> Result<(), Box<dyn std::error::Error>> {
        let rule = AdBlockRule {
            pattern: pattern.clone(),
            is_regex,
            enabled: true,
            created_at: chrono::Utc::now(),
        };
        
        {
            let mut rules = self.rules.lock().unwrap();
            rules.insert(rule);
        }
        
        self.compile_rules()?;
        self.save_rules()?;
        
        Ok(())
    }

    /// Remove a rule
    pub fn remove_rule(&self, pattern: &str) -> bool {
        let mut rules = self.rules.lock().unwrap();
        let removed = rules.remove(&AdBlockRule {
            pattern: pattern.to_string(),
            is_regex: false, // This doesn't matter for comparison
            enabled: false,
            created_at: chrono::Utc::now(),
        });
        
        if removed {
            let _ = self.compile_rules();
            let _ = self.save_rules();
        }
        
        removed
    }

    /// Enable/disable a rule
    pub fn set_rule_enabled(&self, pattern: &str, enabled: bool) -> bool {
        let mut rules = self.rules.lock().unwrap();
        
        if let Some(rule) = rules.get(&AdBlockRule {
            pattern: pattern.to_string(),
            is_regex: false,
            enabled: !enabled, // Look for the opposite state
            created_at: chrono::Utc::now(),
        }).cloned() {
            rules.remove(&rule);
            let mut updated_rule = rule;
            updated_rule.enabled = enabled;
            rules.insert(updated_rule);
            
            let _ = self.compile_rules();
            let _ = self.save_rules();
            true
        } else {
            false
        }
    }

    /// Get all rules
    pub fn get_rules(&self) -> Vec<AdBlockRule> {
        let rules = self.rules.lock().unwrap();
        rules.iter().cloned().collect()
    }

    /// Load predefined filter list
    pub fn load_filter_list(&self, filter_list: FilterList) -> Result<(), Box<dyn std::error::Error>> {
        let rules = match filter_list {
            FilterList::EasyList => self.get_easylist_rules(),
            FilterList::EasyPrivacy => self.get_easyprivacy_rules(),
            FilterList::FanboyAnnoyances => self.get_fanboy_annoyances_rules(),
            FilterList::Custom => vec![], // Custom rules are user-defined
        };
        
        {
            let mut existing_rules = self.rules.lock().unwrap();
            for rule in rules {
                existing_rules.insert(rule);
            }
        }
        
        self.compile_rules()?;
        self.save_rules()?;
        
        Ok(())
    }

    /// Clear all rules
    pub fn clear_rules(&self) {
        self.rules.lock().unwrap().clear();
        self.compiled_regexes.lock().unwrap().clear();
        let _ = self.save_rules();
    }

    /// Get statistics
    pub fn get_stats(&self) -> (usize, usize) {
        let rules = self.rules.lock().unwrap();
        let active_count = rules.iter().filter(|r| r.enabled).count();
        (rules.len(), active_count)
    }

    // Private helper methods
    
    fn compile_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules = self.rules.lock().unwrap();
        let mut regexes = Vec::new();
        
        for rule in rules.iter().filter(|r| r.enabled) {
            let pattern = if rule.is_regex {
                rule.pattern.clone()
            } else {
                // Convert simple pattern to regex
                self.pattern_to_regex(&rule.pattern)
            };
            
            if let Ok(regex) = Regex::new(&pattern) {
                regexes.push(regex);
            }
        }
        
        *self.compiled_regexes.lock().unwrap() = regexes;
        Ok(())
    }
    
    fn pattern_to_regex(&self, pattern: &str) -> String {
        // Escape special regex characters and convert wildcards
        let escaped = regex::escape(pattern);
        escaped.replace(r"\*", ".*").replace(r"\?", ".")
    }
    
    fn save_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let rules = self.rules.lock().unwrap();
        let path = self.config_dir.join("rules.json");
        let content = serde_json::to_string_pretty(&*rules)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn load_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("rules.json");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let rules: HashSet<AdBlockRule> = serde_json::from_str(&content)?;
            
            *self.rules.lock().unwrap() = rules;
            self.compile_rules()?;
        }
        Ok(())
    }
    
    fn add_default_rules(&self) -> Result<(), Box<dyn std::error::Error>> {
        let default_rules = vec![
            // Common ad domains
            ".*\\.doubleclick\\.net.*",
            ".*\\.googlesyndication\\.com.*",
            ".*\\.googleadservices\\.com.*",
            ".*\\.facebook\\.com/tr.*",
            ".*\\.facebook\\.com/impression\\.php.*",
            ".*\\.adservice\\.google\\.com.*",
            
            // Analytics and tracking
            ".*\\.google-analytics\\.com.*",
            ".*\\.analytics\\.google\\.com.*",
            ".*\\.facebook\\.com/tr.*",
            ".*\\.facebook\\.com/pixel.*",
            
            // Popup/popunder ads
            ".*popup.*",
            ".*popunder.*",
            
            // Common ad paths
            ".*/ads/.*",
            ".*/ad/.*",
            ".*/banner/.*",
        ];
        
        {
            let mut rules = self.rules.lock().unwrap();
            for pattern in default_rules {
                rules.insert(AdBlockRule {
                    pattern: pattern.to_string(),
                    is_regex: true,
                    enabled: true,
                    created_at: chrono::Utc::now(),
                });
            }
        }
        
        self.compile_rules()?;
        self.save_rules()?;
        
        Ok(())
    }
    
    fn get_easylist_rules(&self) -> Vec<AdBlockRule> {
        // These would typically be loaded from the actual EasyList
        // For demo purposes, including some common patterns
        vec![
            AdBlockRule {
                pattern: ".*\\.2mdn\\.net.*".to_string(),
                is_regex: true,
                enabled: true,
                created_at: chrono::Utc::now(),
            },
            AdBlockRule {
                pattern: ".*\\.adnxs\\.com.*".to_string(),
                is_regex: true,
                enabled: true,
                created_at: chrono::Utc::now(),
            },
        ]
    }
    
    fn get_easyprivacy_rules(&self) -> Vec<AdBlockRule> {
        vec![
            AdBlockRule {
                pattern: ".*\\.scorecardresearch\\.com.*".to_string(),
                is_regex: true,
                enabled: true,
                created_at: chrono::Utc::now(),
            },
        ]
    }
    
    fn get_fanboy_annoyances_rules(&self) -> Vec<AdBlockRule> {
        vec![
            AdBlockRule {
                pattern: ".*newsletter.*".to_string(),
                is_regex: true,
                enabled: true,
                created_at: chrono::Utc::now(),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ad_blocker_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let blocker = AdBlocker::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test default rules are loaded
        let rules = blocker.get_rules();
        assert!(!rules.is_empty());
        
        // Test blocking functionality
        assert!(blocker.should_block("https://pagead2.googlesyndication.com/pagead/js/adsbygoogle.js"));
        assert!(blocker.should_block("https://www.google-analytics.com/analytics.js"));
        
        // Test non-blocking URLs
        assert!(!blocker.should_block("https://example.com"));
        assert!(!blocker.should_block("https://github.com"));
    }

    #[test]
    fn test_add_remove_rules() {
        let temp_dir = TempDir::new().unwrap();
        let blocker = AdBlocker::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Add a custom rule
        blocker.add_rule(".*test-ad\\.com.*".to_string(), true).unwrap();
        
        // Test the new rule works
        assert!(blocker.should_block("https://test-ad.com/banner.jpg"));
        
        // Remove the rule
        assert!(blocker.remove_rule(".*test-ad\\.com.*"));
        assert!(!blocker.should_block("https://test-ad.com/banner.jpg"));
    }

    #[test]
    fn test_rule_enabling() {
        let temp_dir = TempDir::new().unwrap();
        let blocker = AdBlocker::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Add a rule
        blocker.add_rule(".*temp-block\\.com.*".to_string(), true).unwrap();
        
        // Disable it
        assert!(blocker.set_rule_enabled(".*temp-block\\.com.*", false));
        assert!(!blocker.should_block("https://temp-block.com/ad.js"));
        
        // Re-enable it
        assert!(blocker.set_rule_enabled(".*temp-block\\.com.*", true));
        assert!(blocker.should_block("https://temp-block.com/ad.js"));
    }
}