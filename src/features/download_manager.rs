// Download Manager
use crate::core::{Download, DownloadStatus};
use crate::utils::{filename_from_url, sanitize_filename, format_file_size};
use reqwest::Client;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;
use tokio_stream::StreamExt;

/// Download manager for handling file downloads
pub struct DownloadManager {
    downloads: Arc<Mutex<Vec<Download>>>,
    download_dir: PathBuf,
    client: Client,
    tx: mpsc::UnboundedSender<DownloadEvent>,
    rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<DownloadEvent>>>>,
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
        let (tx, rx) = mpsc::unbounded_channel();
        
        Ok(Self {
            downloads: Arc::new(Mutex::new(Vec::new())),
            download_dir,
            client,
            tx,
            rx: Arc::new(Mutex::new(Some(rx))),
        })
    }

    /// Start a new download
    pub async fn start_download(&self, url: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let filename = sanitize_filename(&filename_from_url(url));
        let filepath = self.download_dir.join(&filename);
        
        // Check if file already exists, append number if needed
        let final_filepath = self.get_unique_filepath(filepath);
        
        let download_id = {
            let mut downloads = self.downloads.lock().unwrap();
            let id = downloads.len() + 1;
            
            downloads.push(Download {
                id,
                url: url.to_string(),
                filename: final_filepath.file_name().unwrap().to_string_lossy().to_string(),
                path: final_filepath.to_string_lossy().to_string(),
                size: 0,
                downloaded: 0,
                status: DownloadStatus::Pending,
                started_at: chrono::Utc::now(),
            });
            
            id
        };
        
        // Start download in background
        self.download_file(download_id, url.to_string(), final_filepath).await?;
        
        Ok(download_id)
    }

    /// Cancel a download
    pub fn cancel_download(&self, download_id: usize) -> bool {
        let mut downloads = self.downloads.lock().unwrap();
        if let Some(download) = downloads.iter_mut().find(|d| d.id == download_id) {
            download.status = DownloadStatus::Cancelled;
            let _ = self.tx.send(DownloadEvent::Cancelled(download_id));
            true
        } else {
            false
        }
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

    /// Remove completed/cancelled download from list
    pub fn remove_download(&self, download_id: usize) -> bool {
        let mut downloads = self.downloads.lock().unwrap();
        let len_before = downloads.len();
        downloads.retain(|d| d.id != download_id);
        downloads.len() != len_before
    }

    /// Clear all completed downloads
    pub fn clear_completed(&self) {
        let mut downloads = self.downloads.lock().unwrap();
        downloads.retain(|d| 
            d.status == DownloadStatus::Downloading || 
            d.status == DownloadStatus::Pending
        );
    }

    /// Get download directory
    pub fn download_dir(&self) -> &Path {
        &self.download_dir
    }

    /// Change download directory
    pub fn set_download_dir(&mut self, new_dir: PathBuf) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&new_dir)?;
        self.download_dir = new_dir;
        Ok(())
    }

    /// Subscribe to download events
    pub fn subscribe_events(&self) -> mpsc::UnboundedReceiver<DownloadEvent> {
        self.rx.lock().unwrap().take().unwrap()
    }

    // Private helper methods
    
    async fn download_file(
        &self,
        download_id: usize,
        url: String,
        filepath: PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = self.client.clone();
        let tx = self.tx.clone();
        let downloads = self.downloads.clone();
        
        tokio::spawn(async move {
            let _ = tx.send(DownloadEvent::Started(download_id));
            
            // Update status to downloading
            {
                let mut downloads = downloads.lock().unwrap();
                if let Some(download) = downloads.iter_mut().find(|d| d.id == download_id) {
                    download.status = DownloadStatus::Downloading;
                }
            }
            
            match client.get(&url).send().await {
                Ok(response) => {
                    let total_size = response.content_length().unwrap_or(0);
                    
                    // Update total size
                    {
                        let mut downloads = downloads.lock().unwrap();
                        if let Some(download) = downloads.iter_mut().find(|d| d.id == download_id) {
                            download.size = total_size;
                        }
                    }
                    
                    let mut file = match File::create(&filepath) {
                        Ok(f) => f,
                        Err(e) => {
                            let _ = tx.send(DownloadEvent::Failed(download_id, e.to_string()));
                            return;
                        }
                    };
                    
                    let mut stream = response.bytes_stream();
                    let mut downloaded: u64 = 0;
                    
                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(chunk) => {
                                if let Err(e) = file.write_all(&chunk) {
                                    let _ = tx.send(DownloadEvent::Failed(download_id, e.to_string()));
                                    return;
                                }
                                
                                downloaded += chunk.len() as u64;
                                
                                // Send progress update
                                let _ = tx.send(DownloadEvent::Progress(download_id, downloaded, total_size));
                                
                                // Update downloaded amount
                                {
                                    let mut downloads = downloads.lock().unwrap();
                                    if let Some(download) = downloads.iter_mut().find(|d| d.id == download_id) {
                                        download.downloaded = downloaded;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = tx.send(DownloadEvent::Failed(download_id, e.to_string()));
                                return;
                            }
                        }
                    }
                    
                    // Mark as completed
                    {
                        let mut downloads = downloads.lock().unwrap();
                        if let Some(download) = downloads.iter_mut().find(|d| d.id == download_id) {
                            download.status = DownloadStatus::Completed;
                            download.downloaded = downloaded;
                        }
                    }
                    
                    let _ = tx.send(DownloadEvent::Completed(download_id));
                }
                Err(e) => {
                    let _ = tx.send(DownloadEvent::Failed(download_id, e.to_string()));
                    
                    // Update status to failed
                    {
                        let mut downloads = downloads.lock().unwrap();
                        if let Some(download) = downloads.iter_mut().find(|d| d.id == download_id) {
                            download.status = DownloadStatus::Failed;
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
    
    fn get_unique_filepath(&self, mut filepath: PathBuf) -> PathBuf {
        let original_path = filepath.clone();
        let mut counter = 1;
        
        while filepath.exists() {
            let stem = original_path.file_stem().unwrap().to_string_lossy();
            let extension = original_path.extension().map(|ext| ext.to_string_lossy());
            
            let new_name = if let Some(ext) = extension {
                format!("{}_{}.{}", stem, counter, ext)
            } else {
                format!("{}_{}", stem, counter)
            };
            
            filepath = original_path.parent().unwrap().join(new_name);
            counter += 1;
        }
        
        filepath
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_download_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let manager = DownloadManager::new(Some(temp_dir.path().to_path_buf())).unwrap();
        
        assert_eq!(manager.download_dir(), temp_dir.path());
        assert_eq!(manager.get_downloads().len(), 0);
    }

    #[test]
    fn test_filename_sanitization() {
        let manager = DownloadManager::new(None).unwrap();
        let downloads = manager.get_downloads();
        assert_eq!(downloads.len(), 0);
    }
}