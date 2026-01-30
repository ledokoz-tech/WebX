// Image Optimization and Lazy Loading
use serde::{Deserialize, Serialize};

/// Image optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOptimizationConfig {
    pub enable_compression: bool,
    pub quality: u8, // 1-100
    pub enable_lazy_loading: bool,
    pub lazy_load_threshold: u32, // pixels from viewport
    pub enable_webp_conversion: bool,
    pub max_width: Option<u32>,
    pub max_height: Option<u32>,
}

impl Default for ImageOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_compression: true,
            quality: 85,
            enable_lazy_loading: true,
            lazy_load_threshold: 300,
            enable_webp_conversion: true,
            max_width: Some(1920),
            max_height: Some(1080),
        }
    }
}

/// Image optimizer for reducing bandwidth usage
pub struct ImageOptimizer {
    config: ImageOptimizationConfig,
}

impl ImageOptimizer {
    /// Create new image optimizer
    pub fn new(config: Option<ImageOptimizationConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
        }
    }

    /// Optimize image data
    pub fn optimize_image(
        &self,
        image_data: &[u8],
        content_type: &str,
    ) -> Result<OptimizedImage, Box<dyn std::error::Error>> {
        if !self.config.enable_compression {
            return Ok(OptimizedImage {
                data: image_data.to_vec(),
                content_type: content_type.to_string(),
                original_size: image_data.len(),
                optimized_size: image_data.len(),
                compression_ratio: 1.0,
            });
        }

        let mut optimized_data = image_data.to_vec();
        let original_size = image_data.len();
        let mut optimized_size = original_size;

        // Convert to WebP if enabled and beneficial
        if self.config.enable_webp_conversion && content_type.starts_with("image/") {
            if let Some(webp_data) = self.convert_to_webp(image_data, content_type)? {
                if webp_data.len() < optimized_data.len() {
                    optimized_data = webp_data;
                    optimized_size = optimized_data.len();
                }
            }
        }

        // Resize if dimensions exceed limits
        if self.config.max_width.is_some() || self.config.max_height.is_some() {
            if let Some(resized_data) = self.resize_image(
                &optimized_data,
                self.config.max_width,
                self.config.max_height,
            )? {
                optimized_data = resized_data;
                optimized_size = optimized_data.len();
            }
        }

        let compression_ratio = optimized_size as f64 / original_size as f64;

        Ok(OptimizedImage {
            data: optimized_data,
            content_type: if compression_ratio < 1.0 && self.config.enable_webp_conversion {
                "image/webp".to_string()
            } else {
                content_type.to_string()
            },
            original_size,
            optimized_size,
            compression_ratio,
        })
    }

    /// Generate lazy loading HTML attributes
    pub fn generate_lazy_attributes(&self, src: &str, alt: &str) -> String {
        if self.config.enable_lazy_loading {
            format!(
                r#"src="data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg'/%3E" data-src="{}" alt="{}" loading="lazy" decoding="async""#,
                src, alt
            )
        } else {
            format!(r#"src="{}" alt="{}""#, src, alt)
        }
    }

    /// Get JavaScript for lazy loading implementation
    pub fn get_lazy_load_script(&self) -> String {
        if !self.config.enable_lazy_loading {
            return String::new();
        }

        format!(
            r#"
(function() {{
    const lazyLoadThreshold = {};
    
    const observer = new IntersectionObserver((entries) => {{
        entries.forEach(entry => {{
            if (entry.isIntersecting) {{
                const img = entry.target;
                const src = img.dataset.src;
                if (src) {{
                    img.src = src;
                    img.removeAttribute('data-src');
                    observer.unobserve(img);
                }}
            }}
        }});
    }}, {{
        rootMargin: `${{lazyLoadThreshold}}px`
    }});
    
    // Observe all lazy images
    document.querySelectorAll('img[data-src]').forEach(img => {{
        observer.observe(img);
    }});
    
    // Handle dynamically added images
    const mutationObserver = new MutationObserver(mutations => {{
        mutations.forEach(mutation => {{
            mutation.addedNodes.forEach(node => {{
                if (node.nodeType === 1) {{ // Element node
                    if (node.tagName === 'IMG' && node.dataset.src) {{
                        observer.observe(node);
                    }}
                    // Check children
                    node.querySelectorAll && node.querySelectorAll('img[data-src]').forEach(img => {{
                        observer.observe(img);
                    }});
                }}
            }});
        }});
    }});
    
    mutationObserver.observe(document.body, {{
        childList: true,
        subtree: true
    }});
}})();
"#,
            self.config.lazy_load_threshold
        )
    }

    /// Get optimization statistics
    pub fn get_stats(&self) -> ImageOptimizationStats {
        // In a real implementation, this would track actual optimizations
        ImageOptimizationStats {
            images_processed: 0,
            total_saved_bytes: 0,
            average_compression_ratio: 1.0,
            webp_conversions: 0,
        }
    }

    // Private helper methods
    
    fn convert_to_webp(
        &self,
        _image_data: &[u8],
        _content_type: &str,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // This would use an actual WebP encoder library
        // For demo purposes, returning None
        Ok(None)
    }

    fn resize_image(
        &self,
        _image_data: &[u8],
        _max_width: Option<u32>,
        _max_height: Option<u32>,
    ) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        // This would use an actual image processing library
        // For demo purposes, returning None
        Ok(None)
    }
}

/// Optimized image result
#[derive(Debug, Clone)]
pub struct OptimizedImage {
    pub data: Vec<u8>,
    pub content_type: String,
    pub original_size: usize,
    pub optimized_size: usize,
    pub compression_ratio: f64,
}

/// Image optimization statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOptimizationStats {
    pub images_processed: u64,
    pub total_saved_bytes: usize,
    pub average_compression_ratio: f64,
    pub webp_conversions: u64,
}