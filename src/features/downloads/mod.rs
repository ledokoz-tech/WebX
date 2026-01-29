// Download Management Module
pub mod manager;
pub mod progress;
pub mod storage;

pub use manager::DownloadManager;
pub use progress::DownloadProgress;
pub use storage::DownloadStorage;

use crate::core::Download;