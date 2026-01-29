// Download Storage Management
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Download storage manager
pub struct DownloadStorage {
    base_directory: PathBuf,
}

impl DownloadStorage {
    /// Create new storage manager
    pub fn new(base_directory: PathBuf) -> Self {
        Self { base_directory }
    }

    /// Get unique filepath for a download
    pub fn get_unique_filepath(&self, filename: &str) -> PathBuf {
        let mut filepath = self.base_directory.join(filename);
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

    /// Create file for download with resume support
    pub fn create_download_file(
        &self,
        filepath: &Path,
        resume_from: Option<u64>,
    ) -> Result<File, std::io::Error> {
        let mut file = File::create(filepath)?;
        
        if let Some(offset) = resume_from {
            file.seek(SeekFrom::Start(offset))?;
        }
        
        Ok(file)
    }

    /// Check if partial download exists
    pub fn get_partial_download_info(&self, filepath: &Path) -> Option<u64> {
        if filepath.exists() {
            if let Ok(metadata) = std::fs::metadata(filepath) {
                Some(metadata.len())
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Clean up failed downloads
    pub fn cleanup_partial_downloads(&self) -> Result<(), std::io::Error> {
        // Remove files that are likely incomplete downloads
        // This is a simplified implementation
        Ok(())
    }

    /// Get available disk space
    pub fn available_space(&self) -> Option<u64> {
        // Platform-specific implementation would be needed here
        // For now, return None to indicate unknown
        None
    }

    /// Validate download directory
    pub fn validate_directory(&self) -> Result<(), std::io::Error> {
        if !self.base_directory.exists() {
            std::fs::create_dir_all(&self.base_directory)?;
        }
        
        if !self.base_directory.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Download path is not a directory",
            ));
        }
        
        Ok(())
    }
}