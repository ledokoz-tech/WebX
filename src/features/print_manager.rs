// Print Manager
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// Print job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrintJobStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// Print settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    pub printer_name: Option<String>,
    pub copies: u32,
    pub page_range: Option<(u32, u32)>,
    pub duplex: bool,
    pub color: bool,
    pub paper_size: PaperSize,
    pub orientation: PageOrientation,
    pub margins: PageMargins,
    pub scale: f32,
}

impl Default for PrintSettings {
    fn default() -> Self {
        Self {
            printer_name: None,
            copies: 1,
            page_range: None,
            duplex: false,
            color: true,
            paper_size: PaperSize::A4,
            orientation: PageOrientation::Portrait,
            margins: PageMargins::default(),
            scale: 1.0,
        }
    }
}

/// Paper size options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaperSize {
    A4,
    Letter,
    Legal,
    A3,
    Custom(f32, f32), // width, height in mm
}

/// Page orientation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PageOrientation {
    Portrait,
    Landscape,
}

/// Page margins
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMargins {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl Default for PageMargins {
    fn default() -> Self {
        Self {
            top: 20.0,
            bottom: 20.0,
            left: 20.0,
            right: 20.0,
        }
    }
}

/// Printer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterInfo {
    pub name: String,
    pub is_default: bool,
    pub is_network: bool,
    pub status: PrinterStatus,
    pub supported_paper_sizes: Vec<PaperSize>,
    pub can_duplex: bool,
    pub can_color: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PrinterStatus {
    Ready,
    Busy,
    Error,
    Offline,
    Unknown,
}

/// Print job information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintJob {
    pub id: usize,
    pub title: String,
    pub url: String,
    pub pages: u32,
    pub settings: PrintSettings,
    pub status: PrintJobStatus,
    pub progress: f32, // 0.0 to 1.0
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Print manager for handling print jobs
pub struct PrintManager {
    jobs: Arc<Mutex<Vec<PrintJob>>>,
    printers: Arc<Mutex<Vec<PrinterInfo>>>,
    next_job_id: Arc<Mutex<usize>>,
    config_dir: PathBuf,
}

impl PrintManager {
    /// Create a new print manager
    pub fn new(config_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let config_dir = config_dir.unwrap_or_else(|| {
            let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
            path.push("webx");
            path.push("print");
            path
        });
        
        // Create config directory
        std::fs::create_dir_all(&config_dir)?;
        
        let manager = Self {
            jobs: Arc::new(Mutex::new(Vec::new())),
            printers: Arc::new(Mutex::new(Vec::new())),
            next_job_id: Arc::new(Mutex::new(1)),
            config_dir,
        };
        
        // Discover available printers
        manager.discover_printers()?;
        
        Ok(manager)
    }

    /// Print a web page
    pub async fn print_page(
        &self,
        url: &str,
        title: &str,
        settings: PrintSettings,
    ) -> Result<usize, Box<dyn std::error::Error>> {
        let job_id = {
            let mut next_id = self.next_job_id.lock().unwrap();
            let id = *next_id;
            *next_id += 1;
            id
        };
        
        let job = PrintJob {
            id: job_id,
            title: title.to_string(),
            url: url.to_string(),
            pages: 1, // Would be determined from content
            settings,
            status: PrintJobStatus::Pending,
            progress: 0.0,
            created_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        };
        
        {
            let mut jobs = self.jobs.lock().unwrap();
            jobs.push(job);
        }
        
        // Start print job in background
        self.process_print_job(job_id).await?;
        
        Ok(job_id)
    }

    /// Get print job status
    pub fn get_job_status(&self, job_id: usize) -> Option<PrintJob> {
        let jobs = self.jobs.lock().unwrap();
        jobs.iter().find(|job| job.id == job_id).cloned()
    }

    /// Cancel a print job
    pub fn cancel_job(&self, job_id: usize) -> bool {
        let mut jobs = self.jobs.lock().unwrap();
        if let Some(job) = jobs.iter_mut().find(|job| job.id == job_id) {
            if job.status == PrintJobStatus::Pending || job.status == PrintJobStatus::Processing {
                job.status = PrintJobStatus::Cancelled;
                job.completed_at = Some(chrono::Utc::now());
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Get all print jobs
    pub fn get_jobs(&self) -> Vec<PrintJob> {
        self.jobs.lock().unwrap().clone()
    }

    /// Get active print jobs
    pub fn get_active_jobs(&self) -> Vec<PrintJob> {
        let jobs = self.jobs.lock().unwrap();
        jobs.iter()
            .filter(|job| {
                job.status == PrintJobStatus::Pending || job.status == PrintJobStatus::Processing
            })
            .cloned()
            .collect()
    }

    /// Get available printers
    pub fn get_printers(&self) -> Vec<PrinterInfo> {
        self.printers.lock().unwrap().clone()
    }

    /// Get default printer
    pub fn get_default_printer(&self) -> Option<PrinterInfo> {
        let printers = self.printers.lock().unwrap();
        printers.iter().find(|printer| printer.is_default).cloned()
    }

    /// Set default printer
    pub fn set_default_printer(&self, printer_name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut printers = self.printers.lock().unwrap();
        for printer in printers.iter_mut() {
            printer.is_default = printer.name == printer_name;
        }
        self.save_printer_config()?;
        Ok(())
    }

    /// Generate print preview HTML
    pub fn generate_print_preview(&self, html_content: &str, settings: &PrintSettings) -> String {
        let css = self.generate_print_css(settings);
        
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>Print Preview</title>
    <style>
        {}
    </style>
</head>
<body>
    <div class="print-content">
        {}
    </div>
</body>
</html>"#,
            css, html_content
        )
    }

    /// Get JavaScript for print integration
    pub fn get_print_script(&self) -> String {
        r#"
(function() {
    const printManager = {
        printPage: function(options) {
            window.ipc.send({
                type: 'print-page',
                url: window.location.href,
                title: document.title,
                options: options || {}
            });
        },
        
        printPreview: function() {
            window.ipc.send({
                type: 'print-preview',
                html: document.documentElement.outerHTML
            });
        },
        
        getPrinters: function() {
            return new Promise((resolve) => {
                window.ipc.send({
                    type: 'get-printers'
                });
                // Would listen for response
                setTimeout(() => resolve([]), 100);
            });
        }
    };
    
    // Expose to global scope
    window.printManager = printManager;
    
    // Override default print
    window.print = function() {
        printManager.printPage();
    };
})();
"#
        .to_string()
    }

    // Private helper methods
    
    async fn process_print_job(&self, job_id: usize) -> Result<(), Box<dyn std::error::Error>> {
        // Update job status to processing
        {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = PrintJobStatus::Processing;
                job.started_at = Some(chrono::Utc::now());
                job.progress = 0.1;
            }
        }
        
        // Simulate print processing
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // Update progress
        {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.progress = 0.8;
            }
        }
        
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        
        // Complete the job
        {
            let mut jobs = self.jobs.lock().unwrap();
            if let Some(job) = jobs.iter_mut().find(|j| j.id == job_id) {
                job.status = PrintJobStatus::Completed;
                job.progress = 1.0;
                job.completed_at = Some(chrono::Utc::now());
            }
        }
        
        Ok(())
    }
    
    fn discover_printers(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut printers = self.printers.lock().unwrap();
        printers.clear();
        
        // In a real implementation, you would query the system for available printers
        // For demo, add some mock printers
        printers.push(PrinterInfo {
            name: "PDF Printer".to_string(),
            is_default: true,
            is_network: false,
            status: PrinterStatus::Ready,
            supported_paper_sizes: vec![PaperSize::A4, PaperSize::Letter],
            can_duplex: true,
            can_color: true,
        });
        
        printers.push(PrinterInfo {
            name: "Network Printer".to_string(),
            is_default: false,
            is_network: true,
            status: PrinterStatus::Ready,
            supported_paper_sizes: vec![PaperSize::A4, PaperSize::Letter, PaperSize::Legal],
            can_duplex: true,
            can_color: true,
        });
        
        self.save_printer_config()?;
        Ok(())
    }
    
    fn generate_print_css(&self, settings: &PrintSettings) -> String {
        let (width, height) = match settings.paper_size {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::Letter => (216.0, 279.0),
            PaperSize::Legal => (216.0, 356.0),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::Custom(w, h) => (w, h),
        };
        
        let (page_width, page_height) = if settings.orientation == PageOrientation::Landscape {
            (height, width)
        } else {
            (width, height)
        };
        
        format!(
            r#"
@page {{
    size: {}mm {}mm;
    margin: {}mm {}mm {}mm {}mm;
    @bottom-center {{
        content: "Page " counter(page);
    }}
}}

body {{
    font-family: Arial, sans-serif;
    font-size: 12pt;
    line-height: 1.4;
    color: black;
    background: white;
}}

.print-content {{
    width: {}mm;
    min-height: {}mm;
}}

@media print {{
    body {{
        margin: 0;
        padding: 0;
    }}
    
    .print-content {{
        margin: 0;
        padding: 0;
    }}
}}
"#,
            page_width,
            page_height,
            settings.margins.top,
            settings.margins.right,
            settings.margins.bottom,
            settings.margins.left,
            page_width - settings.margins.left - settings.margins.right,
            page_height - settings.margins.top - settings.margins.bottom
        )
    }
    
    fn save_printer_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = self.config_dir.join("printers.json");
        let printers = self.printers.lock().unwrap();
        let content = serde_json::to_string_pretty(&*printers)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_print_manager_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let manager = PrintManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        // Test printer discovery
        let printers = manager.get_printers();
        assert!(!printers.is_empty());
        assert!(printers.iter().any(|p| p.is_default));
        
        // Test print job creation
        let settings = PrintSettings::default();
        let job_id = manager
            .print_page("https://example.com", "Test Page", settings)
            .await
            .unwrap();
        
        assert!(job_id > 0);
        
        // Test job status
        let job = manager.get_job_status(job_id).unwrap();
        assert_eq!(job.id, job_id);
        assert_eq!(job.title, "Test Page");
        
        // Test getting all jobs
        let jobs = manager.get_jobs();
        assert!(!jobs.is_empty());
        assert_eq!(jobs.len(), 1);
    }

    #[test]
    fn test_print_settings() {
        let mut settings = PrintSettings::default();
        settings.copies = 2;
        settings.duplex = true;
        settings.paper_size = PaperSize::Letter;
        
        let manager = PrintManager::new(None).unwrap();
        let preview_html = manager.generate_print_preview("<p>Test content</p>", &settings);
        
        assert!(preview_html.contains("@page"));
        assert!(preview_html.contains("size: 216mm 279mm")); // Letter size
    }

    #[tokio::test]
    async fn test_print_job_cancellation() {
        let manager = PrintManager::new(None).unwrap();
        let settings = PrintSettings::default();
        let job_id = manager
            .print_page("https://example.com", "Test Page", settings)
            .await
            .unwrap();
        
        // Cancel the job
        assert!(manager.cancel_job(job_id));
        
        // Check status
        let job = manager.get_job_status(job_id).unwrap();
        assert_eq!(job.status, PrintJobStatus::Cancelled);
    }
}