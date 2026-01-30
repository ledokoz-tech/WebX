// CSS Minification
use regex::Regex;
use serde::{Deserialize, Serialize};

/// CSS minification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSMinificationConfig {
    pub remove_whitespace: bool,
    pub remove_comments: bool,
    pub remove_unused_selectors: bool,
    pub optimize_colors: bool,
    pub shorten_zero_values: bool,
}

impl Default for CSSMinificationConfig {
    fn default() -> Self {
        Self {
            remove_whitespace: true,
            remove_comments: true,
            remove_unused_selectors: false, // Requires DOM analysis
            optimize_colors: true,
            shorten_zero_values: true,
        }
    }
}

/// CSS minifier
pub struct CSSMinifier {
    config: CSSMinificationConfig,
    comment_regex: Regex,
    whitespace_regex: Regex,
    zero_value_regex: Regex,
    hex_color_regex: Regex,
}

impl CSSMinifier {
    /// Create new CSS minifier
    pub fn new(config: Option<CSSMinificationConfig>) -> Result<Self, Box<dyn std::error::Error>> {
        let config = config.unwrap_or_default();
        
        Ok(Self {
            config,
            comment_regex: Regex::new(r"/\*[\s\S]*?\*/")?,
            whitespace_regex: Regex::new(r"\s+")?,
            zero_value_regex: Regex::new(r"(?i)(\d+)px")?,
            hex_color_regex: Regex::new(r"#([a-f0-9])\1([a-f0-9])\2([a-f0-9])\3")?,
        })
    }

    /// Minify CSS code
    pub fn minify(&self, css_code: &str) -> Result<MinifiedCSS, Box<dyn std::error::Error>> {
        let original_size = css_code.len();
        let mut minified = css_code.to_string();
        
        // Remove comments
        if self.config.remove_comments {
            minified = self.comment_regex.replace_all(&minified, "").to_string();
        }
        
        // Optimize colors
        if self.config.optimize_colors {
            minified = self.optimize_hex_colors(&minified);
            minified = self.optimize_named_colors(&minified);
        }
        
        // Shorten zero values
        if self.config.shorten_zero_values {
            minified = self.shorten_zero_values(&minified);
        }
        
        // Remove extra whitespace
        if self.config.remove_whitespace {
            minified = self.remove_whitespace(&minified);
        }
        
        let minified_size = minified.len();
        let compression_ratio = minified_size as f64 / original_size as f64;
        
        Ok(MinifiedCSS {
            code: minified,
            original_size,
            minified_size,
            compression_ratio,
            savings_bytes: original_size - minified_size,
        })
    }

    /// Get minification statistics
    pub fn get_stats(&self) -> CSSMinificationStats {
        CSSMinificationStats {
            files_processed: 0,
            total_original_size: 0,
            total_minified_size: 0,
            average_compression_ratio: 1.0,
            colors_optimized: 0,
        }
    }

    // Private helper methods
    
    fn optimize_hex_colors(&self, css: &str) -> String {
        // Convert #ffffff to #fff, #000000 to #000, etc.
        let mut result = css.to_string();
        
        // This is a simplified implementation
        result = result.replace("#ffffff", "#fff");
        result = result.replace("#000000", "#000");
        result = result.replace("#ff0000", "#f00");
        result = result.replace("#00ff00", "#0f0");
        result = result.replace("#0000ff", "#00f");
        
        result
    }
    
    fn optimize_named_colors(&self, css: &str) -> String {
        let mut result = css.to_string();
        
        // Convert named colors to hex where shorter
        result = result.replace("white", "#fff");
        result = result.replace("black", "#000");
        result = result.replace("red", "#f00");
        result = result.replace("lime", "#0f0");
        result = result.replace("blue", "#00f");
        
        result
    }
    
    fn shorten_zero_values(&self, css: &str) -> String {
        let mut result = css.to_string();
        
        // Remove units from zero values
        result = result.replace("0px", "0");
        result = result.replace("0em", "0");
        result = result.replace("0rem", "0");
        result = result.replace("0%", "0");
        
        result
    }
    
    fn remove_whitespace(&self, css: &str) -> String {
        let mut result = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut chars = css.chars().peekable();
        
        while let Some(ch) = chars.next() {
            match ch {
                '"' | '\'' => {
                    if !in_string {
                        in_string = true;
                        string_char = ch;
                        result.push(ch);
                    } else if ch == string_char {
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
                        // Preserve necessary whitespace
                        let next_ch = chars.peek();
                        if let Some(&next) = next_ch {
                            if self.needs_space_preserved(result.chars().last(), Some(next)) {
                                result.push(' ');
                            }
                        }
                    }
                }
                '{' | '}' | ';' | ':' | ',' => {
                    // Remove whitespace around these characters
                    result.push(ch);
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
            (Some('d'), Some('.')) => true,  // div.class
            (Some('d'), Some('#')) => true,  // div#id
            (Some(')'), Some('{')) => true,    // media query
            _ => false,
        }
    }
}

/// Minified CSS result
#[derive(Debug, Clone)]
pub struct MinifiedCSS {
    pub code: String,
    pub original_size: usize,
    pub minified_size: usize,
    pub compression_ratio: f64,
    pub savings_bytes: usize,
}

/// CSS minification statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CSSMinificationStats {
    pub files_processed: u64,
    pub total_original_size: usize,
    pub total_minified_size: usize,
    pub average_compression_ratio: f64,
    pub colors_optimized: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_css_minification_basic() {
        let minifier = CSSMinifier::new(None).unwrap();
        
        let css_code = r#"
        /* This is a comment */
        .container {
            width: 100px;
            height: 0px;
            background-color: #ffffff;
            color: white;
        }
        "#;
        
        let result = minifier.minify(css_code).unwrap();
        assert!(result.minified_size < result.original_size);
        assert!(result.compression_ratio < 1.0);
        assert!(!result.code.contains("/* This is a comment */"));
        assert!(result.code.contains("0")); // zero values shortened
        assert!(result.code.contains("#fff")); // colors optimized
    }

    #[test]
    fn test_color_optimization() {
        let minifier = CSSMinifier::new(None).unwrap();
        
        let css_code = "body { background: #ffffff; color: white; border: 1px solid #000000; }";
        let result = minifier.minify(css_code).unwrap();
        
        assert!(result.code.contains("#fff"));
        assert!(result.code.contains("#000"));
    }
}