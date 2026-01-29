// Bandwidth Usage Monitoring
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Bandwidth monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthConfig {
    pub enable_monitoring: bool,
    pub sampling_interval_ms: u64,
    pub history_duration_minutes: u64,
    pub alert_threshold_kbps: Option<u64>,
}

impl Default for BandwidthConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            sampling_interval_ms: 1000,
            history_duration_minutes: 60,
            alert_threshold_kbps: Some(1000), // 1 Mbps
        }
    }
}

/// Bandwidth usage data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthSample {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

/// Bandwidth monitor
pub struct BandwidthMonitor {
    config: BandwidthConfig,
    samples: Arc<Mutex<Vec<BandwidthSample>>>,
    last_sample: Arc<Mutex<Option<BandwidthSample>>>,
    total_received: Arc<Mutex<u64>>,
    total_sent: Arc<Mutex<u64>>,
    start_time: Instant,
}

impl BandwidthMonitor {
    /// Create new bandwidth monitor
    pub fn new(config: Option<BandwidthConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            samples: Arc::new(Mutex::new(Vec::new())),
            last_sample: Arc::new(Mutex::new(None)),
            total_received: Arc::new(Mutex::new(0)),
            total_sent: Arc::new(Mutex::new(0)),
            start_time: Instant::now(),
        }
    }

    /// Record data transfer
    pub fn record_transfer(&self, bytes_received: u64, bytes_sent: u64) {
        if !self.config.enable_monitoring {
            return;
        }

        let timestamp = chrono::Utc::now();
        
        // Update totals
        *self.total_received.lock().unwrap() += bytes_received;
        *self.total_sent.lock().unwrap() += bytes_sent;

        // Create new sample
        let sample = BandwidthSample {
            timestamp,
            bytes_received,
            bytes_sent,
        };

        // Update last sample
        *self.last_sample.lock().unwrap() = Some(sample.clone());

        // Add to samples collection
        {
            let mut samples = self.samples.lock().unwrap();
            samples.push(sample);
            
            // Trim old samples
            self.trim_old_samples(&mut samples);
        }
    }

    /// Get current bandwidth usage (last second)
    pub fn get_current_bandwidth(&self) -> BandwidthUsage {
        let samples = self.samples.lock().unwrap();
        
        if samples.is_empty() {
            return BandwidthUsage::default();
        }

        // Get samples from last 1 second
        let cutoff = chrono::Utc::now() - chrono::Duration::seconds(1);
        let recent_samples: Vec<&BandwidthSample> = samples
            .iter()
            .filter(|sample| sample.timestamp > cutoff)
            .collect();

        if recent_samples.is_empty() {
            return BandwidthUsage::default();
        }

        let total_rx: u64 = recent_samples.iter().map(|s| s.bytes_received).sum();
        let total_tx: u64 = recent_samples.iter().map(|s| s.bytes_sent).sum();

        BandwidthUsage {
            download_kbps: (total_rx as f64 * 8.0 / 1000.0) as u64,
            upload_kbps: (total_tx as f64 * 8.0 / 1000.0) as u64,
            samples_count: recent_samples.len(),
        }
    }

    /// Get average bandwidth over time period
    pub fn get_average_bandwidth(&self, duration_minutes: u64) -> BandwidthUsage {
        let samples = self.samples.lock().unwrap();
        
        if samples.is_empty() {
            return BandwidthUsage::default();
        }

        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(duration_minutes as i64);
        let period_samples: Vec<&BandwidthSample> = samples
            .iter()
            .filter(|sample| sample.timestamp > cutoff)
            .collect();

        if period_samples.is_empty() {
            return BandwidthUsage::default();
        }

        let total_rx: u64 = period_samples.iter().map(|s| s.bytes_received).sum();
        let total_tx: u64 = period_samples.iter().map(|s| s.bytes_sent).sum();
        let seconds = duration_minutes * 60;

        BandwidthUsage {
            download_kbps: (total_rx as f64 * 8.0 / 1000.0 / seconds as f64) as u64,
            upload_kbps: (total_tx as f64 * 8.0 / 1000.0 / seconds as f64) as u64,
            samples_count: period_samples.len(),
        }
    }

    /// Get total data transferred
    pub fn get_total_transferred(&self) -> DataTransferStats {
        let received = *self.total_received.lock().unwrap();
        let sent = *self.total_sent.lock().unwrap();
        let elapsed = self.start_time.elapsed().as_secs();

        DataTransferStats {
            total_received_bytes: received,
            total_sent_bytes: sent,
            total_bytes: received + sent,
            session_duration_seconds: elapsed,
            average_download_kbps: if elapsed > 0 {
                (received as f64 * 8.0 / 1000.0 / elapsed as f64) as u64
            } else {
                0
            },
            average_upload_kbps: if elapsed > 0 {
                (sent as f64 * 8.0 / 1000.0 / elapsed as f64) as u64
            } else {
                0
            },
        }
    }

    /// Check if bandwidth threshold is exceeded
    pub fn is_bandwidth_exceeded(&self) -> bool {
        if let Some(threshold) = self.config.alert_threshold_kbps {
            let current = self.get_current_bandwidth();
            current.download_kbps + current.upload_kbps > threshold
        } else {
            false
        }
    }

    /// Get bandwidth history for charting
    pub fn get_bandwidth_history(&self, minutes: u64) -> Vec<BandwidthHistoryPoint> {
        let samples = self.samples.lock().unwrap();
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(minutes as i64);
        
        samples
            .iter()
            .filter(|sample| sample.timestamp > cutoff)
            .map(|sample| BandwidthHistoryPoint {
                timestamp: sample.timestamp,
                download_kbps: (sample.bytes_received as f64 * 8.0 / 1000.0) as u64,
                upload_kbps: (sample.bytes_sent as f64 * 8.0 / 1000.0) as u64,
            })
            .collect()
    }

    /// Reset counters
    pub fn reset(&self) {
        *self.total_received.lock().unwrap() = 0;
        *self.total_sent.lock().unwrap() = 0;
        self.samples.lock().unwrap().clear();
        *self.last_sample.lock().unwrap() = None;
        // Note: start_time is not reset to maintain session continuity
    }

    /// Get configuration
    pub fn get_config(&self) -> &BandwidthConfig {
        &self.config
    }

    /// Set configuration
    pub fn set_config(&mut self, config: BandwidthConfig) {
        self.config = config;
    }

    // Private helper methods
    
    fn trim_old_samples(&self, samples: &mut Vec<BandwidthSample>) {
        let cutoff = chrono::Utc::now() - chrono::Duration::minutes(self.config.history_duration_minutes as i64);
        samples.retain(|sample| sample.timestamp > cutoff);
    }
}

/// Current bandwidth usage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BandwidthUsage {
    pub download_kbps: u64,
    pub upload_kbps: u64,
    pub samples_count: usize,
}

/// Total data transfer statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransferStats {
    pub total_received_bytes: u64,
    pub total_sent_bytes: u64,
    pub total_bytes: u64,
    pub session_duration_seconds: u64,
    pub average_download_kbps: u64,
    pub average_upload_kbps: u64,
}

/// Bandwidth history data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthHistoryPoint {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub download_kbps: u64,
    pub upload_kbps: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bandwidth_monitoring() {
        let monitor = BandwidthMonitor::new(None);
        
        // Record some transfers
        monitor.record_transfer(1024 * 1024, 512 * 1024); // 1MB received, 512KB sent
        monitor.record_transfer(2 * 1024 * 1024, 1024 * 1024); // 2MB received, 1MB sent
        
        // Test total transfer stats
        let stats = monitor.get_total_transferred();
        assert_eq!(stats.total_received_bytes, 3 * 1024 * 1024);
        assert_eq!(stats.total_sent_bytes, 1536 * 1024);
        
        // Test current bandwidth (should be 0 since no recent samples)
        let current = monitor.get_current_bandwidth();
        assert_eq!(current.download_kbps, 0);
        assert_eq!(current.upload_kbps, 0);
        
        // Test average bandwidth over session
        assert!(stats.average_download_kbps > 0);
        assert!(stats.average_upload_kbps > 0);
    }

    #[test]
    fn test_bandwidth_reset() {
        let monitor = BandwidthMonitor::new(None);
        
        monitor.record_transfer(1000, 500);
        let stats_before = monitor.get_total_transferred();
        assert!(stats_before.total_bytes > 0);
        
        monitor.reset();
        let stats_after = monitor.get_total_transferred();
        assert_eq!(stats_after.total_bytes, 0);
    }
}