// WebX Enhanced Features Module

// Organized modules
pub mod tabs;
pub mod downloads;
pub mod security;
pub mod ui;
pub mod productivity;
pub mod system;

// Legacy re-exports for backward compatibility
pub mod tab_manager {
    pub use crate::features::tabs::*;
}
pub mod download_manager {
    pub use crate::features::downloads::*;
}
pub mod password_manager {
    pub use crate::features::security::password_manager::*;
}
pub mod ad_blocker {
    pub use crate::features::security::ad_blocker::*;
}
pub mod reading_mode {
    pub use crate::features::ui::reader::*;
}
pub mod dark_mode {
    pub use crate::features::ui::themes::*;
}
pub mod spell_checker {
    pub use crate::features::ui::spell_checker::*;
}
pub mod pdf_viewer {
    pub use crate::features::productivity::pdf::*;
}
pub mod print_manager {
    pub use crate::features::productivity::printing::*;
}
pub mod find_in_page {
    pub use crate::features::ui::search::*;
}
pub mod session_restore {
    pub use crate::features::productivity::session::*;
}
pub mod keyboard_shortcuts {
    pub use crate::features::system::shortcuts::*;
}
pub mod privacy_protection {
    pub use crate::features::security::privacy::*;
}
pub mod user_agent_switcher {
    pub use crate::features::system::user_agent::*;
}
pub mod proxy_manager {
    pub use crate::features::system::proxy::*;
}

// New advanced features
pub mod caching;
pub mod resource_optimizer;
pub mod web_inspector;
pub mod extension_system;
pub mod smart_address_bar;
pub mod history_manager;
pub mod bookmark_manager;
pub mod sandbox;
pub mod certificate_manager;

pub use tabs::*;
pub use downloads::*;
pub use security::*;
pub use ui::*;
pub use productivity::*;
pub use system::*;

// Re-export legacy items
pub use tab_manager::*;
pub use download_manager::*;
pub use password_manager::*;
pub use ad_blocker::*;
pub use reading_mode::*;
pub use dark_mode::*;
pub use spell_checker::*;
pub use pdf_viewer::*;
pub use print_manager::*;
pub use find_in_page::*;
pub use session_restore::*;
pub use keyboard_shortcuts::*;
pub use privacy_protection::*;
pub use user_agent_switcher::*;
pub use proxy_manager::*;