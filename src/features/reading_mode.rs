// Reading Mode for Articles
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

/// Reading mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingModeConfig {
    pub font_family: String,
    pub font_size: u32,
    pub line_height: f32,
    pub text_color: String,
    pub background_color: String,
    pub width: u32,
    pub margin: u32,
}

impl Default for ReadingModeConfig {
    fn default() -> Self {
        Self {
            font_family: "Georgia, serif".to_string(),
            font_size: 18,
            line_height: 1.6,
            text_color: "#333333".to_string(),
            background_color: "#ffffff".to_string(),
            width: 800,
            margin: 40,
        }
    }
}

/// Article content extracted by reading mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArticleContent {
    pub title: String,
    pub author: Option<String>,
    pub publish_date: Option<String>,
    pub content: String,
    pub excerpt: String,
    pub image: Option<String>,
}

/// Reading mode extractor for web pages
pub struct ReadingMode {
    config: ReadingModeConfig,
}

impl ReadingMode {
    /// Create a new reading mode extractor
    pub fn new(config: Option<ReadingModeConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// Extract article content from HTML
    pub fn extract_article(&self, html: &str, url: &str) -> Option<ArticleContent> {
        let document = Html::parse_document(html);
        
        // Try to find the main content
        let content = self.find_main_content(&document)?;
        
        // Extract metadata
        let title = self.extract_title(&document, url);
        let author = self.extract_author(&document);
        let publish_date = self.extract_publish_date(&document);
        let image = self.extract_image(&document, url);
        let excerpt = self.generate_excerpt(&content);
        
        Some(ArticleContent {
            title,
            author,
            publish_date,
            content,
            excerpt,
            image,
        })
    }

    /// Generate reading-friendly HTML
    pub fn generate_reader_html(&self, article: &ArticleContent) -> String {
        let css = self.generate_css();
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>{css}</style>
</head>
<body>
    <article class="reader-content">
        <header class="reader-header">
            <h1>{title}</h1>
            {metadata}
        </header>
        <div class="reader-body">
            {content}
        </div>
    </article>
</body>
</html>"#,
            title = article.title,
            metadata = self.format_metadata(article),
            content = article.content,
            css = css
        )
    }

    /// Update configuration
    pub fn set_config(&mut self, config: ReadingModeConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ReadingModeConfig {
        &self.config
    }

    // Private helper methods
    
    fn find_main_content(&self, document: &Html) -> Option<String> {
        // Try common selectors for article content
        let selectors = vec![
            "article",
            ".article-content",
            ".post-content",
            ".entry-content",
            ".content",
            "[role='main']",
            ".main-content",
            "#main-content",
            ".story-body",
            ".article-body",
        ];
        
        for selector_str in selectors {
            let selector = Selector::parse(selector_str).ok()?;
            if let Some(element) = document.select(&selector).next() {
                let content = element.inner_html();
                if self.is_content_sufficient(&content) {
                    return Some(self.clean_content(&content));
                }
            }
        }
        
        // Fallback: look for paragraphs
        self.extract_from_paragraphs(document)
    }
    
    fn extract_title(&self, document: &Html, url: &str) -> String {
        // Try various title selectors
        let title_selectors = vec![
            "title",
            "h1",
            "h1 a",
            ".entry-title",
            ".post-title",
            ".article-title",
        ];
        
        for selector_str in title_selectors {
            let selector = match Selector::parse(selector_str) {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            if let Some(element) = document.select(&selector).next() {
                let title = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !title.is_empty() && title.len() < 200 {
                    return title;
                }
            }
        }
        
        // Fallback to URL
        self.title_from_url(url)
    }
    
    fn extract_author(&self, document: &Html) -> Option<String> {
        let author_selectors = vec![
            "[rel='author']",
            ".author",
            ".byline",
            ".post-author",
            "[class*='author']",
        ];
        
        for selector_str in author_selectors {
            let selector = Selector::parse(selector_str).ok()?;
            if let Some(element) = document.select(&selector).next() {
                let author = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !author.is_empty() && author.len() < 100 {
                    return Some(author);
                }
            }
        }
        
        None
    }
    
    fn extract_publish_date(&self, document: &Html) -> Option<String> {
        let date_selectors = vec![
            "time[datetime]",
            ".publish-date",
            ".post-date",
            "[class*='date']",
        ];
        
        for selector_str in date_selectors {
            let selector = Selector::parse(selector_str).ok()?;
            if let Some(element) = document.select(&selector).next() {
                if let Some(datetime) = element.value().attr("datetime") {
                    return Some(datetime.to_string());
                }
                
                let date_text = element.text().collect::<Vec<_>>().join(" ").trim().to_string();
                if !date_text.is_empty() {
                    return Some(date_text);
                }
            }
        }
        
        None
    }
    
    fn extract_image(&self, document: &Html, base_url: &str) -> Option<String> {
        // Look for featured/open graph images
        let img_selectors = vec![
            "meta[property='og:image']",
            "meta[name='twitter:image']",
            ".featured-image img",
            "article img:first-of-type",
        ];
        
        for selector_str in img_selectors {
            let selector = Selector::parse(selector_str).ok()?;
            if let Some(element) = document.select(&selector).next() {
                if let Some(img_url) = element.value().attr("content").or(element.value().attr("src")) {
                    return Some(self.resolve_url(img_url, base_url));
                }
            }
        }
        
        None
    }
    
    fn extract_from_paragraphs(&self, document: &Html) -> Option<String> {
        let p_selector = Selector::parse("p").ok()?;
        let paragraphs: Vec<String> = document
            .select(&p_selector)
            .map(|p| p.inner_html())
            .filter(|content| content.len() > 50)
            .take(20)
            .collect();
        
        if paragraphs.len() >= 3 {
            Some(paragraphs.join("\n\n"))
        } else {
            None
        }
    }
    
    fn is_content_sufficient(&self, content: &str) -> bool {
        // Basic heuristics to determine if content is substantial
        let text_length = content.chars().count();
        let paragraph_count = content.matches("<p").count();
        let word_count = content.split_whitespace().count();
        
        text_length > 500 && paragraph_count >= 2 && word_count > 100
    }
    
    fn clean_content(&self, content: &str) -> String {
        // Remove common unwanted elements
        let unwanted_selectors = vec![
            "script",
            "style",
            "noscript",
            ".ad",
            "[class*='advertisement']",
            "[id*='ad-']",
            ".social-share",
            ".comments",
        ];
        
        let mut cleaned = content.to_string();
        
        // This is a simplified cleaning - in practice you'd use a proper HTML parser
        for selector in unwanted_selectors {
            // Remove elements matching selector (simplified)
            let pattern = format!(r"<{}[^>]*>.*?</{}>", selector, selector);
            if let Ok(re) = regex::Regex::new(&pattern) {
                cleaned = re.replace_all(&cleaned, "").to_string();
            }
        }
        
        cleaned
    }
    
    fn generate_excerpt(&self, content: &str) -> String {
        // Extract first few sentences as excerpt
        let text_only = scraper::Html::parse_fragment(content)
            .root_element()
            .text()
            .collect::<Vec<_>>()
            .join(" ");
        
        let sentences: Vec<&str> = text_only.split(|c| c == '.' || c == '!' || c == '?')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .take(2)
            .collect();
        
        if sentences.is_empty() {
            text_only.chars().take(200).collect()
        } else {
            sentences.join(". ") + "."
        }
    }
    
    fn title_from_url(&self, url: &str) -> String {
        // Extract domain and path to create a title
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                let domain = host.replace("www.", "");
                return format!("Article from {}", domain);
            }
        }
        "Web Article".to_string()
    }
    
    fn format_metadata(&self, article: &ArticleContent) -> String {
        let mut metadata = String::new();
        
        if let Some(author) = &article.author {
            metadata.push_str(&format!("<p class=\"reader-author\">By {}</p>", author));
        }
        
        if let Some(date) = &article.publish_date {
            metadata.push_str(&format!("<p class=\"reader-date\">Published: {}</p>", date));
        }
        
        metadata
    }
    
    fn generate_css(&self) -> String {
        format!(
            r#"
body {{
    font-family: {};
    font-size: {}px;
    line-height: {};
    color: {};
    background-color: {};
    margin: 0;
    padding: 20px;
}}

.reader-content {{
    max-width: {}px;
    margin: 0 auto;
    padding: {}px;
}}

.reader-header h1 {{
    font-size: 2em;
    margin-bottom: 20px;
    line-height: 1.2;
}}

.reader-author, .reader-date {{
    color: #666;
    font-size: 0.9em;
    margin: 5px 0;
}}

.reader-body {{
    margin-top: 30px;
}}

.reader-body p {{
    margin-bottom: 1.2em;
    text-align: justify;
}}

img {{
    max-width: 100%;
    height: auto;
    display: block;
    margin: 20px auto;
}}

@media (max-width: 768px) {{
    .reader-content {{
        padding: 10px;
    }}
    
    body {{
        font-size: {}px;
    }}
}}
"#,
            self.config.font_family,
            self.config.font_size,
            self.config.line_height,
            self.config.text_color,
            self.config.background_color,
            self.config.width,
            self.config.margin,
            (self.config.font_size as f32 * 0.9) as u32
        )
    }
    
    fn resolve_url(&self, url: &str, base_url: &str) -> String {
        if url.starts_with("http") {
            url.to_string()
        } else if let Ok(base) = url::Url::parse(base_url) {
            if let Ok(resolved) = base.join(url) {
                resolved.to_string()
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reading_mode_extraction() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head><title>Test Article</title></head>
        <body>
            <article>
                <h1>Test Article Title</h1>
                <p class="author">John Doe</p>
                <p>First paragraph of content...</p>
                <p>Second paragraph with more content...</p>
                <p>Third paragraph concluding the article.</p>
            </article>
        </body>
        </html>
        "#;
        
        let reading_mode = ReadingMode::new(None);
        let article = reading_mode.extract_article(html, "https://example.com/article");
        
        assert!(article.is_some());
        let article = article.unwrap();
        assert_eq!(article.title, "Test Article Title");
        assert_eq!(article.author, Some("John Doe".to_string()));
        assert!(!article.content.is_empty());
    }

    #[test]
    fn test_reading_mode_config() {
        let mut config = ReadingModeConfig::default();
        config.font_size = 20;
        config.background_color = "#f5f5f5".to_string();
        
        let mut reading_mode = ReadingMode::new(Some(config));
        let current_config = reading_mode.get_config();
        
        assert_eq!(current_config.font_size, 20);
        assert_eq!(current_config.background_color, "#f5f5f5");
    }
}