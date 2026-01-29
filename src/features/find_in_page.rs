// Find in Page Feature
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Find options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindOptions {
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub regex: bool,
    pub highlight_all: bool,
}

impl Default for FindOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_words: false,
            regex: false,
            highlight_all: true,
        }
    }
}

/// Find result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindResult {
    pub index: usize,
    pub text: String,
    pub position: (usize, usize), // (start, end) character positions
    pub page_number: Option<u32>,
}

/// Find session
#[derive(Debug, Clone)]
pub struct FindSession {
    pub query: String,
    pub options: FindOptions,
    pub results: Vec<FindResult>,
    pub current_index: usize,
    pub content_hash: u64, // To detect content changes
}

/// Find in page manager
pub struct FindInPage {
    sessions: Arc<Mutex<HashMap<String, FindSession>>>,
    max_results: usize,
}

impl FindInPage {
    /// Create a new find in page manager
    pub fn new(max_results: Option<usize>) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            max_results: max_results.unwrap_or(1000),
        }
    }

    /// Start a new find session
    pub fn start_find(
        &self,
        session_id: &str,
        content: &str,
        query: &str,
        options: FindOptions,
    ) -> Vec<FindResult> {
        let results = self.perform_find(content, query, &options);
        let content_hash = self.hash_content(content);
        
        let session = FindSession {
            query: query.to_string(),
            options,
            results: results.clone(),
            current_index: 0,
            content_hash,
        };
        
        {
            let mut sessions = self.sessions.lock().unwrap();
            sessions.insert(session_id.to_string(), session);
        }
        
        results
    }

    /// Find next occurrence
    pub fn find_next(&self, session_id: &str) -> Option<FindResult> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.results.is_empty() {
                session.current_index = (session.current_index + 1) % session.results.len();
                Some(session.results[session.current_index].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Find previous occurrence
    pub fn find_previous(&self, session_id: &str) -> Option<FindResult> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.results.is_empty() {
                if session.current_index == 0 {
                    session.current_index = session.results.len() - 1;
                } else {
                    session.current_index -= 1;
                }
                Some(session.results[session.current_index].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Jump to specific result
    pub fn jump_to_result(&self, session_id: &str, index: usize) -> Option<FindResult> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            if index < session.results.len() {
                session.current_index = index;
                Some(session.results[index].clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Update find session with new content
    pub fn update_content(&self, session_id: &str, new_content: &str) -> Option<Vec<FindResult>> {
        let mut sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get_mut(session_id) {
            let new_hash = self.hash_content(new_content);
            
            // Only re-find if content actually changed
            if new_hash != session.content_hash {
                let results = self.perform_find(new_content, &session.query, &session.options);
                session.results = results.clone();
                session.content_hash = new_hash;
                session.current_index = 0;
                Some(results)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// End find session
    pub fn end_find(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.lock().unwrap();
        sessions.remove(session_id).is_some()
    }

    /// Get current session info
    pub fn get_session_info(&self, session_id: &str) -> Option<(usize, usize)> {
        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(session_id) {
            Some((session.current_index, session.results.len()))
        } else {
            None
        }
    }

    /// Get all results for a session
    pub fn get_all_results(&self, session_id: &str) -> Option<Vec<FindResult>> {
        let sessions = self.sessions.lock().unwrap();
        sessions.get(session_id).map(|s| s.results.clone())
    }

    /// Generate JavaScript for find in page integration
    pub fn get_find_script(&self) -> String {
        r#"
(function() {
    class WebXFindInPage {
        constructor() {
            this.sessionId = null;
            this.isActive = false;
            this.results = [];
            this.currentIndex = 0;
            this.highlightElements = [];
        }
        
        startFind(query, options = {}) {
            this.cleanupHighlights();
            
            const defaultOptions = {
                caseSensitive: false,
                wholeWords: false,
                regex: false,
                highlightAll: true
            };
            
            const findOptions = { ...defaultOptions, ...options };
            
            window.ipc.send({
                type: 'find-start',
                sessionId: this.generateSessionId(),
                query: query,
                options: findOptions
            });
            
            this.isActive = true;
        }
        
        findNext() {
            if (!this.isActive) return;
            
            window.ipc.send({
                type: 'find-next',
                sessionId: this.sessionId
            });
        }
        
        findPrevious() {
            if (!this.isActive) return;
            
            window.ipc.send({
                type: 'find-previous',
                sessionId: this.sessionId
            });
        }
        
        updateResults(results, currentIndex) {
            this.results = results;
            this.currentIndex = currentIndex;
            this.highlightResults(results);
        }
        
        highlightResults(results) {
            this.cleanupHighlights();
            
            results.forEach((result, index) => {
                // In a real implementation, you would highlight the text
                // This is a simplified example
                console.log(`Highlighting result ${index}: ${result.text}`);
            });
        }
        
        cleanupHighlights() {
            // Remove all highlight elements
            this.highlightElements.forEach(el => el.remove());
            this.highlightElements = [];
        }
        
        endFind() {
            this.cleanupHighlights();
            this.isActive = false;
            this.results = [];
            this.currentIndex = 0;
            
            if (this.sessionId) {
                window.ipc.send({
                    type: 'find-end',
                    sessionId: this.sessionId
                });
                this.sessionId = null;
            }
        }
        
        generateSessionId() {
            return 'find_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9);
        }
        
        getStatus() {
            return {
                isActive: this.isActive,
                resultCount: this.results.length,
                currentIndex: this.currentIndex
            };
        }
    }
    
    // Create global instance
    window.webxFind = new WebXFindInPage();
    
    // Keyboard shortcuts
    document.addEventListener('keydown', function(e) {
        // Ctrl+F or Cmd+F to start find
        if ((e.ctrlKey || e.metaKey) && e.key === 'f') {
            e.preventDefault();
            // Would show find UI
            window.webxFind.startFind('');
        }
        
        // ESC to close find
        if (e.key === 'Escape' && window.webxFind.isActive) {
            e.preventDefault();
            window.webxFind.endFind();
        }
        
        // Enter/Shift+Enter for next/previous
        if (window.webxFind.isActive) {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                window.webxFind.findNext();
            } else if (e.key === 'Enter' && e.shiftKey) {
                e.preventDefault();
                window.webxFind.findPrevious();
            }
        }
    });
})();
"#
        .to_string()
    }

    /// Generate CSS for highlighting
    pub fn get_highlight_css(&self) -> String {
        r#"
.webx-find-highlight {
    background-color: #ffeb3b;
    color: #000;
    padding: 1px 2px;
    border-radius: 2px;
    box-shadow: 0 0 0 1px rgba(0,0,0,0.1);
}

.webx-find-highlight-current {
    background-color: #ff9800;
    color: #fff;
    font-weight: bold;
}

.webx-find-highlight:focus {
    outline: 2px solid #2196f3;
    outline-offset: 1px;
}
"#
        .to_string()
    }

    // Private helper methods
    
    fn perform_find(&self, content: &str, query: &str, options: &FindOptions) -> Vec<FindResult> {
        if query.is_empty() {
            return Vec::new();
        }
        
        let search_text = if options.case_sensitive {
            content.to_string()
        } else {
            content.to_lowercase()
        };
        
        let search_query = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };
        
        let mut results = Vec::new();
        let mut start_pos = 0;
        
        while results.len() < self.max_results {
            let pos = if options.regex {
                self.find_regex(&search_text, &search_query, start_pos, options)
            } else {
                self.find_literal(&search_text, &search_query, start_pos, options)
            };
            
            match pos {
                Some((start, end)) => {
                    // Get the original case text
                    let original_text = content[start..end].to_string();
                    
                    results.push(FindResult {
                        index: results.len(),
                        text: original_text,
                        position: (start, end),
                        page_number: None, // Would be set for paginated content
                    });
                    
                    start_pos = end;
                }
                None => break,
            }
        }
        
        results
    }
    
    fn find_literal(
        &self,
        text: &str,
        query: &str,
        start_pos: usize,
        options: &FindOptions,
    ) -> Option<(usize, usize)> {
        let text_chars: Vec<char> = text.chars().collect();
        let query_chars: Vec<char> = query.chars().collect();
        
        for i in start_pos..text_chars.len() {
            if i + query_chars.len() > text_chars.len() {
                break;
            }
            
            let mut match_found = true;
            for j in 0..query_chars.len() {
                if text_chars[i + j] != query_chars[j] {
                    match_found = false;
                    break;
                }
            }
            
            if match_found {
                // Check word boundaries if needed
                if options.whole_words {
                    let start_boundary = i == 0 || !text_chars[i - 1].is_alphanumeric();
                    let end_boundary = i + query_chars.len() == text_chars.len() 
                        || !text_chars[i + query_chars.len()].is_alphanumeric();
                    
                    if !start_boundary || !end_boundary {
                        continue;
                    }
                }
                
                let start_byte = text[..i].chars().map(|c| c.len_utf8()).sum();
                let match_len: usize = text_chars[i..i + query_chars.len()]
                    .iter()
                    .map(|c| c.len_utf8())
                    .sum();
                
                return Some((start_byte, start_byte + match_len));
            }
        }
        
        None
    }
    
    fn find_regex(
        &self,
        text: &str,
        pattern: &str,
        start_pos: usize,
        _options: &FindOptions,
    ) -> Option<(usize, usize)> {
        if let Ok(regex) = Regex::new(pattern) {
            let slice = &text[start_pos..];
            if let Some(mat) = regex.find(slice) {
                let start = start_pos + mat.start();
                let end = start_pos + mat.end();
                Some((start, end))
            } else {
                None
            }
        } else {
            // Fall back to literal search if regex is invalid
            self.find_literal(text, pattern, start_pos, _options)
        }
    }
    
    fn hash_content(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_literal_search() {
        let finder = FindInPage::new(None);
        let content = "The quick brown fox jumps over the lazy dog. The fox is quick.";
        
        let options = FindOptions::default();
        let results = finder.start_find("session1", content, "fox", options);
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].text, "fox");
        assert_eq!(results[1].text, "fox");
        
        // Test case sensitivity
        let mut case_options = FindOptions::default();
        case_options.case_sensitive = true;
        let case_results = finder.start_find("session2", content, "Fox", case_options);
        assert_eq!(case_results.len(), 0); // No matches with exact case
        
        // Test whole words
        let mut word_options = FindOptions::default();
        word_options.whole_words = true;
        let word_results = finder.start_find("session3", content, "the", word_options);
        assert_eq!(word_results.len(), 2); // Only whole word matches
    }

    #[test]
    fn test_find_navigation() {
        let finder = FindInPage::new(None);
        let content = "test test test test";
        
        let options = FindOptions::default();
        finder.start_find("session1", content, "test", options);
        
        // Test navigation
        let first = finder.find_next("session1").unwrap();
        assert_eq!(first.index, 1); // Second occurrence (0-indexed)
        
        let second = finder.find_next("session1").unwrap();
        assert_eq!(second.index, 2);
        
        let prev = finder.find_previous("session1").unwrap();
        assert_eq!(prev.index, 1);
        
        // Test jumping to specific result
        let jump_result = finder.jump_to_result("session1", 3).unwrap();
        assert_eq!(jump_result.index, 3);
    }

    #[test]
    fn test_find_with_updates() {
        let finder = FindInPage::new(None);
        let content1 = "original content with test word";
        let content2 = "updated content with test word and another test";
        
        let options = FindOptions::default();
        finder.start_find("session1", content1, "test", options.clone());
        
        // Update with new content
        let new_results = finder.update_content("session1", content2).unwrap();
        assert_eq!(new_results.len(), 2); // Now has 2 matches
        
        // Session info should reflect new count
        let info = finder.get_session_info("session1").unwrap();
        assert_eq!(info.1, 2); // 2 total results
    }

    #[test]
    fn test_regex_search() {
        let finder = FindInPage::new(None);
        let content = "Phone: 123-456-7890, Another: 987-654-3210";
        
        let mut options = FindOptions::default();
        options.regex = true;
        let results = finder.start_find("session1", content, r"\d{3}-\d{3}-\d{4}", options);
        
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].text, "123-456-7890");
        assert_eq!(results[1].text, "987-654-3210");
    }
}