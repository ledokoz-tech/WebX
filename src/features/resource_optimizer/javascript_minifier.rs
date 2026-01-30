// JavaScript Minification
use regex::Regex;
use serde::{Deserialize, Serialize};

/// JavaScript minification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSMinificationConfig {
    pub remove_whitespace: bool,
    pub remove_comments: bool,
    pub shorten_variable_names: bool,
    pub remove_console_logs: bool,
    pub optimize_booleans: bool,
}

impl Default for JSMinificationConfig {
    fn default() -> Self {
        Self {
            remove_whitespace: true,
            remove_comments: true,
            shorten_variable_names: false, // More aggressive, potential issues
            remove_console_logs: true,
            optimize_booleans: true,
        }
    }
}

/// JavaScript minifier
pub struct JavaScriptMinifier {
    config: JSMinificationConfig,
    comment_regex: Regex,
    console_log_regex: Regex,
    whitespace_regex: Regex,
}

impl JavaScriptMinifier {
    /// Create new JavaScript minifier
    pub fn new(config: Option<JSMinificationConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        
        Ok(Self {
            config,
            comment_regex: Regex::new(r"/\*[\s\S]*?\*/|//.*")?,
            console_log_regex: Regex::new(r"console\.(log|debug|info|warn|error)\([^)]*\);?")?,
            whitespace_regex: Regex::new(r"\s+")?,
        })
    }

    /// Minify JavaScript code
    pub fn minify(&self, js_code: &str) -> Result<MinifiedJS, Box<dyn std::error::Error>> {
        let original_size = js_code.len();
        let mut minified = js_code.to_string();
        
        // Remove comments
        if self.config.remove_comments {
            minified = self.comment_regex.replace_all(&minified, "").to_string();
        }
        
        // Remove console logs
        if self.config.remove_console_logs {
            minified = self.console_log_regex.replace_all(&minified, "").to_string();
        }
        
        // Optimize boolean literals
        if self.config.optimize_booleans {
            minified = minified.replace("true", "!0");
            minified = minified.replace("false", "!1");
        }
        
        // Remove extra whitespace
        if self.config.remove_whitespace {
            minified = self.remove_whitespace(&minified);
        }
        
        let minified_size = minified.len();
        let compression_ratio = minified_size as f64 / original_size as f64;
        
        Ok(MinifiedJS {
            code: minified,
            original_size,
            minified_size,
            compression_ratio,
            savings_bytes: original_size - minified_size,
        })
    }

    /// Get minification statistics
    pub fn get_stats(&self) -> JSMinificationStats {
        JSMinificationStats {
            files_processed: 0,
            total_original_size: 0,
            total_minified_size: 0,
            average_compression_ratio: 1.0,
        }
    }

    // Private helper methods
    
    fn remove_whitespace(&self, code: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut chars = code.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' => {
                    if !in_string {
                        in_string = true;
                        string_char = ch;
                        result.push(ch);
                    } else if ch == string_char {
                        // Check for escaped quote
                        let prev_is_escape = result.chars().last() == Some('\\');
                        if !prev_is_escape {
                            in_string = false;
                        }
                        result.push(ch);
                    } else {
                        result.push(ch);
                    }
                }
                ' ' | '\t' | '\n' | '\r' => {
                    if in_string {
                        result.push(ch);
                    } else {
                        // Only preserve necessary whitespace
                        let next_ch = chars.peek();
                        if let Some(&next) = next_ch {
                            // Preserve space between certain tokens
                            if self.needs_space_preserved(result.chars().last(), Some(next)) {
                                result.push(' ');
                            }
                        }
                    }
                }
                _ => {
                    result.push(ch);
                }
            }
        }
        
        result
    }
    
    fn needs_space_preserved(&self, prev: Option<char>, next: Option<char>) -> bool {
        match (prev, next) {
            (Some("return"), Some(c)) if c.is_alphabetic() => true,
            (Some(c), Some("function")) if c.is_alphabetic() => true,
            (Some("if"), Some('(')) => true,
            (Some("for"), Some('(')) => true,
            (Some("while"), Some('(')) => true,
            (Some("catch"), Some('(')) => true,
            _ => false,
        }
    }
}

/// Minified JavaScript result
#[derive(Debug, Clone)]
pub struct MinifiedJS {
    pub code: String,
    pub original_size: usize,
    pub minified_size: usize,
    pub compression_ratio: f64,
    pub savings_bytes: usize,
}

/// JavaScript minification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSMinificationStats {
    pub files_processed: u64,
    pub total_original_size: usize,
    pub total_minified_size: usize,
    pub average_compression_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_minification_basic() {
        let minifier = JavaScriptMinifier::new(None).unwrap();
        
        let js_code = r#"
        // This is a comment
        function helloWorld() {
            console.log("Hello, World!");
            return true;
        }
        "#;
        
        let result = minifier.minify(js_code).unwrap();
        assert!(result.minified_size < result.original_size);
        assert!(result.compression_ratio < 1.0);
        assert!(!result.code.contains("// This is a comment"));
        assert!(!result.code.contains("console.log"));
        assert!(result.code.contains("!0")); // true optimized to !0
    }

    #[test]
    fn test_whitespace_removal() {
        let minifier = JavaScriptMinifier::new(None).unwrap();
        
        let js_code = "var   x   =   5 ; \n\n function test ( ) { return x ; }";
        let result = minifier.minify(js_code).unwrap();
        
        // Should remove extra spaces and newlines
        assert!(!result.code.contains("   "));
        assert!(!result.code.contains("\n\n"));
    }
}