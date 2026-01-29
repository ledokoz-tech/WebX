// Spell Checker for Text Inputs
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Supported languages for spell checking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SpellLanguage {
    EnglishUS,
    EnglishUK,
    Spanish,
    French,
    German,
    Italian,
    Portuguese,
    Russian,
}

impl SpellLanguage {
    pub fn code(&self) -> &'static str {
        match self {
            SpellLanguage::EnglishUS => "en-US",
            SpellLanguage::EnglishUK => "en-GB",
            SpellLanguage::Spanish => "es",
            SpellLanguage::French => "fr",
            SpellLanguage::German => "de",
            SpellLanguage::Italian => "it",
            SpellLanguage::Portuguese => "pt",
            SpellLanguage::Russian => "ru",
        }
    }
    
    pub fn name(&self) -> &'static str {
        match self {
            SpellLanguage::EnglishUS => "English (US)",
            SpellLanguage::EnglishUK => "English (UK)",
            SpellLanguage::Spanish => "Spanish",
            SpellLanguage::French => "French",
            SpellLanguage::German => "German",
            SpellLanguage::Italian => "Italian",
            SpellLanguage::Portuguese => "Portuguese",
            SpellLanguage::Russian => "Russian",
        }
    }
}

/// Spell checker dictionary
pub struct SpellDictionary {
    words: HashSet<String>,
    language: SpellLanguage,
}

impl SpellDictionary {
    pub fn new(language: SpellLanguage) -> Self {
        Self {
            words: HashSet::new(),
            language,
        }
    }
    
    pub fn add_word(&mut self, word: &str) {
        self.words.insert(word.to_lowercase());
    }
    
    pub fn remove_word(&mut self, word: &str) -> bool {
        self.words.remove(&word.to_lowercase())
    }
    
    pub fn contains(&self, word: &str) -> bool {
        self.words.contains(&word.to_lowercase())
    }
    
    pub fn get_language(&self) -> &SpellLanguage {
        &self.language
    }
    
    pub fn word_count(&self) -> usize {
        self.words.len()
    }
}

/// Spell checker service
pub struct SpellChecker {
    dictionaries: Arc<Mutex<std::collections::HashMap<SpellLanguage, SpellDictionary>>>,
    active_languages: Arc<Mutex<Vec<SpellLanguage>>>,
    user_dictionary: Arc<Mutex<HashSet<String>>>,
    config_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellCheckResult {
    pub word: String,
    pub offset: usize,
    pub suggestions: Vec<String>,
    pub is_misspelled: bool,
}

impl SpellChecker {
    /// Create a new spell checker
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("spellcheck");
            path
        });
        
        // Create config directory
        fs::create_dir_all(&config_dir)?;
        
        let spell_checker = Self {
            dictionaries: Arc::new(Mutex::new(std::collections::HashMap::new())),
            active_languages: Arc::new(Mutex::new(vec![SpellLanguage::EnglishUS])),
            user_dictionary: Arc::new(Mutex::new(HashSet::new())),
            config_dir,
        };
        
        // Load user dictionary
        spell_checker.load_user_dictionary()?;
        
        // Load default dictionaries
        spell_checker.load_default_dictionaries()?;
        
        Ok(spell_checker)
    }

    /// Check spelling of text
    pub fn check_text(&self, text: &str) -> Vec<SpellCheckResult> {
        let mut results = Vec::new();
        let words = self.extract_words(text);
        
        let dictionaries = self.dictionaries.lock().unwrap();
        let user_dict = self.user_dictionary.lock().unwrap();
        let active_langs = self.active_languages.lock().unwrap();
        
        for (word, offset) in words {
            if word.len() < 2 || word.chars().all(|c| c.is_ascii_digit()) {
                continue; // Skip very short words and numbers
            }
            
            let is_correct = user_dict.contains(&word.to_lowercase())
                || active_langs.iter().any(|lang| {
                    dictionaries
                        .get(lang)
                        .map(|dict| dict.contains(&word))
                        .unwrap_or(false)
                });
            
            if !is_correct {
                let suggestions = self.get_suggestions(&word, &dictionaries, &active_langs);
                results.push(SpellCheckResult {
                    word: word.clone(),
                    offset,
                    suggestions,
                    is_misspelled: true,
                });
            }
        }
        
        results
    }

    /// Add word to user dictionary
    pub fn add_to_user_dictionary(&self, word: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut user_dict = self.user_dictionary.lock().unwrap();
            user_dict.insert(word.to_lowercase());
        }
        
        self.save_user_dictionary()?;
        Ok(())
    }

    /// Remove word from user dictionary
    pub fn remove_from_user_dictionary(&self, word: &str) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut user_dict = self.user_dictionary.lock().unwrap();
            user_dict.remove(&word.to_lowercase());
        }
        
        self.save_user_dictionary()?;
        Ok(())
    }

    /// Set active languages
    pub fn set_active_languages(&self, languages: Vec<SpellLanguage>) -> Result<(), Box<dyn std::error::Error>> {
        *self.active_languages.lock().unwrap() = languages;
        self.save_config()?;
        Ok(())
    }

    /// Get active languages
    pub fn get_active_languages(&self) -> Vec<SpellLanguage> {
        self.active_languages.lock().unwrap().clone()
    }

    /// Get available languages
    pub fn get_available_languages(&self) -> Vec<SpellLanguage> {
        vec![
            SpellLanguage::EnglishUS,
            SpellLanguage::EnglishUK,
            SpellLanguage::Spanish,
            SpellLanguage::French,
            SpellLanguage::German,
            SpellLanguage::Italian,
            SpellLanguage::Portuguese,
            SpellLanguage::Russian,
        ]
    }

    /// Get user dictionary words
    pub fn get_user_words(&self) -> Vec<String> {
        let user_dict = self.user_dictionary.lock().unwrap();
        user_dict.iter().cloned().collect()
    }

    /// Clear user dictionary
    pub fn clear_user_dictionary(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.user_dictionary.lock().unwrap().clear();
        self.save_user_dictionary()?;
        Ok(())
    }

    /// Get JavaScript for spell checking integration
    pub fn get_spell_check_script(&self) -> String {
        format!(
            r#"
(function() {{
    const spellChecker = {{
        checkText: function(text) {{
            // This would communicate with the Rust backend
            // For now, we'll simulate the response
            return [];
        }},
        
        addToDictionary: function(word) {{
            window.ipc.send({{
                type: 'spellcheck-add-word',
                word: word
            }});
        }},
        
        getSuggestions: function(word) {{
            // Simulated suggestions
            return ['suggestion1', 'suggestion2'];
        }}
    }};
    
    // Expose to global scope
    window.spellChecker = spellChecker;
    
    // Monitor text inputs for spell checking
    document.addEventListener('input', function(e) {{
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {{
            // Debounced spell checking would go here
        }}
    }});
}})();
"#
        )
    }

    // Private helper methods
    
    fn extract_words(&self, text: &str) -> Vec<(String, usize)> {
        let mut words = Vec::new();
        let mut current_word = String::new();
        let mut start_pos = 0;
        
        for (i, ch) in text.char_indices() {
            if ch.is_alphabetic() || ch == '\'' {
                if current_word.is_empty() {
                    start_pos = i;
                }
                current_word.push(ch);
            } else if !current_word.is_empty() {
                words.push((current_word.clone(), start_pos));
                current_word.clear();
            }
        }
        
        // Don't forget the last word
        if !current_word.is_empty() {
            words.push((current_word, start_pos));
        }
        
        words
    }
    
    fn get_suggestions(
        &self,
        word: &str,
        dictionaries: &std::collections::HashMap<SpellLanguage, SpellDictionary>,
        active_langs: &[SpellLanguage],
    ) -> Vec<String> {
        let mut suggestions = Vec::new();
        let word_lower = word.to_lowercase();
        
        // Simple edit distance suggestions (would be more sophisticated in practice)
        for lang in active_langs {
            if let Some(dict) = dictionaries.get(lang) {
                for dict_word in dict.words.iter() {
                    if self.edit_distance(&word_lower, dict_word) <= 2 {
                        suggestions.push(dict_word.clone());
                        if suggestions.len() >= 5 {
                            break;
                        }
                    }
                }
            }
            if suggestions.len() >= 5 {
                break;
            }
        }
        
        suggestions.truncate(5);
        suggestions
    }
    
    fn edit_distance(&self, s1: &str, s2: &str) -> usize {
        // Levenshtein distance implementation
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();
        
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,
                        matrix[i][j - 1] + 1,
                    ),
                    matrix[i - 1][j - 1] + cost,
                );
            }
        }
        
        matrix[len1][len2]
    }
    
    fn load_user_dictionary(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("user_dictionary.txt");
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            let mut user_dict = self.user_dictionary.lock().unwrap();
            for word in content.lines() {
                let trimmed = word.trim();
                if !trimmed.is_empty() {
                    user_dict.insert(trimmed.to_lowercase());
                }
            }
        }
        Ok(())
    }
    
    fn save_user_dictionary(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("user_dictionary.txt");
        let user_dict = self.user_dictionary.lock().unwrap();
        let content = user_dict.iter().cloned().collect::<Vec<_>>().join("\n");
        fs::write(path, content)?;
        Ok(())
    }
    
    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("config.json");
        let active_langs = self.active_languages.lock().unwrap();
        let config = serde_json::json!({
            "active_languages": *active_langs
        });
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(path, content)?;
        Ok(())
    }
    
    fn load_default_dictionaries(&self) -> Result<(), Box<dyn std::error::Error>> {
        // In a real implementation, you would load actual dictionary files
        // For demo purposes, we'll create small sample dictionaries
        
        let mut dicts = self.dictionaries.lock().unwrap();
        
        // Sample English dictionary
        let mut english_dict = SpellDictionary::new(SpellLanguage::EnglishUS);
        let common_words = vec![
            "the", "be", "to", "of", "and", "a", "in", "that", "have", "i",
            "it", "for", "not", "on", "with", "he", "as", "you", "do", "at",
            "this", "but", "his", "by", "from", "they", "we", "say", "her", "she",
            "or", "an", "will", "my", "one", "all", "would", "there", "their",
            "what", "so", "up", "out", "if", "about", "who", "get", "which", "go",
            "me", "when", "make", "can", "like", "time", "no", "just", "him", "know",
            "take", "people", "into", "year", "your", "good", "some", "could", "them",
            "see", "other", "than", "then", "now", "look", "only", "come", "its",
            "over", "think", "also", "back", "after", "use", "two", "how", "our",
            "work", "first", "well", "way", "even", "new", "want", "because", "any",
            "these", "give", "day", "most", "us", "is", "was", "are", "were", "been",
            "being", "have", "has", "had", "having", "do", "does", "did", "doing",
            "will", "would", "shall", "should", "may", "might", "must", "can", "could",
        ];
        
        for word in common_words {
            english_dict.add_word(word);
        }
        
        dicts.insert(SpellLanguage::EnglishUS, english_dict);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_spell_checker_basic_functionality() {
        let temp_dir = TempDir::new().unwrap();
        let checker = SpellChecker::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test checking text with misspelled words
        let results = checker.check_text("This is a tst with mispelled wrds.");
        assert!(!results.is_empty());
        
        // Test user dictionary
        checker.add_to_user_dictionary("tst").unwrap();
        let results = checker.check_text("This is a tst.");
        assert!(results.is_empty()); // Should be no errors now
        
        // Test removing from user dictionary
        checker.remove_from_user_dictionary("tst").unwrap();
        let results = checker.check_text("This is a tst.");
        assert!(!results.is_empty()); // Should show error again
    }

    #[test]
    fn test_language_management() {
        let checker = SpellChecker::new(None).unwrap();
        
        // Test available languages
        let languages = checker.get_available_languages();
        assert!(!languages.is_empty());
        assert!(languages.contains(&SpellLanguage::EnglishUS));
        
        // Test setting active languages
        checker.set_active_languages(vec![SpellLanguage::Spanish]).unwrap();
        let active = checker.get_active_languages();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0], SpellLanguage::Spanish);
    }

    #[test]
    fn test_edit_distance() {
        let checker = SpellChecker::new(None).unwrap();
        
        // Test edit distances
        assert_eq!(checker.edit_distance("cat", "bat"), 1);
        assert_eq!(checker.edit_distance("kitten", "sitting"), 3);
        assert_eq!(checker.edit_distance("hello", "hello"), 0);
    }
}