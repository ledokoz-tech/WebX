// Security Features Module
pub mod password_manager;
pub mod ad_blocker;
pub mod privacy;

pub use password_manager::PasswordManager;
pub use ad_blocker::AdBlocker;
pub use privacy::PrivacyProtection;