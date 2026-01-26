// Utility functions for the browser
use std::path::Path;

/// Extract domain from URL
pub fn extract_domain(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(host) = parsed.host_str() {
            return host.replace("www.", "");
        }
    }
    "New Tab".to_string()
}

/// Check if URL is secure (HTTPS)
pub fn is_secure_url(url: &str) -> bool {
    url.starts_with("https://")
}

/// Format file size for display
pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Get filename from URL
pub fn filename_from_url(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.last() {
                if !last.is_empty() {
                    return last.to_string();
                }
            }
        }
    }
    "download".to_string()
}

/// Sanitize filename for safe file system usage
pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            _ => c,
        })
        .collect()
}

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    Path::new(path).exists()
}

/// Get file extension
pub fn get_file_extension(filename: &str) -> Option<String> {
    Path::new(filename)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

/// Validate URL format
pub fn is_valid_url(url: &str) -> bool {
    url::Url::parse(url).is_ok()
}

/// Format timestamp for display
pub fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let local = timestamp.with_timezone(&chrono::Local);
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Truncate string to max length with ellipsis
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
