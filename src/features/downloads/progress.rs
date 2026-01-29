// Download Progress Tracking
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Download progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub download_id: usize,
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
    pub start_time: Instant,
    pub last_update: Instant,
    pub speed_bytes_per_sec: f64,
    pub estimated_time_remaining: Option<f64>, // seconds
}

impl DownloadProgress {
    /// Create new progress tracker
    pub fn new(download_id: usize, total_bytes: u64) -> Self {
        let now = Instant::now();
        Self {
            download_id,
            total_bytes,
            downloaded_bytes: 0,
            start_time: now,
            last_update: now,
            speed_bytes_per_sec: 0.0,
            estimated_time_remaining: None,
        }
    }

    /// Update progress with new downloaded bytes
    pub fn update_progress(&mut self, new_bytes: u64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_update).as_secs_f64();
        
        if elapsed > 0.0 {
            self.speed_bytes_per_sec = (new_bytes - self.downloaded_bytes) as f64 / elapsed;
        }
        
        self.downloaded_bytes = new_bytes;
        self.last_update = now;
        
        // Calculate estimated time remaining
        if self.speed_bytes_per_sec > 0.0 && self.total_bytes > 0 {
            let remaining_bytes = self.total_bytes - self.downloaded_bytes;
            self.estimated_time_remaining = Some(remaining_bytes as f64 / self.speed_bytes_per_sec);
        }
    }

    /// Get completion percentage (0.0 to 100.0)
    pub fn completion_percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            0.0
        } else {
            (self.downloaded_bytes as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Get elapsed time in seconds
    pub fn elapsed_seconds(&self) -> f64 {
        self.last_update.duration_since(self.start_time).as_secs_f64()
    }

    /// Get average speed since start
    pub fn average_speed(&self) -> f64 {
        if self.elapsed_seconds() > 0.0 {
            self.downloaded_bytes as f64 / self.elapsed_seconds()
        } else {
            0.0
        }
    }

    /// Check if download is complete
    pub fn is_complete(&self) -> bool {
        self.downloaded_bytes >= self.total_bytes && self.total_bytes > 0
    }
}