// System Integration Module
pub mod shortcuts;
pub mod proxy;
pub mod user_agent;

// Re-export for convenience
pub use shortcuts::*;
pub use proxy::*;
pub use user_agent::*;