// Download Manager Core
use crate::core::{Download, DownloadStatus};
use reqwest::Client;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Download manager for handling file downloads
pub struct DownloadManager {
    downloads: Arc<Mutex<Vec<Download>>>,
    download_dir: PathBuf,
    client: Client,
    tx: mpsc::UnboundedSender<DownloadEvent>,
}

#[derive(Debug, Clone)]
pub enum DownloadEvent {
    Started(usize),
    Progress(usize, u64, u64), // id, downloaded, total
    Completed(usize),
    Failed(usize, String),
    Cancelled(usize),
}

impl DownloadManager {
    /// Create a new download manager
    pub fn new(download_dir: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let download_dir = download_dir.unwrap_or_else(|| {
            dirs::download_dir().unwrap_or_else(|| PathBuf::from("./downloads"))
        });
        
        // Create download directory if it doesn't exist
        std::fs::create_dir_all(&download_dir)?;
        
        let client = Client::new();
        let (tx, _) = mpsc::unbounded_channel();
        
        Ok(Self {
            downloads: Arc::new(Mutex::new(Vec::new())),
            download_dir,
            client,
            tx,
        })
    }

    /// Get all downloads
    pub fn get_downloads(&self) -> Vec<Download> {
        self.downloads.lock().unwrap().clone()
    }

    /// Get download by ID
    pub fn get_download(&self, download_id: usize) -> Option<Download> {
        self.downloads
            .lock()
            .unwrap()
            .iter()
            .find(|d| d.id == download_id)
            .cloned()
    }

    /// Get download directory
    pub fn download_dir(&self) -> &PathBuf {
        &self.download_dir
    }
}