// Tab Management Module
pub mod manager;
pub mod ui;
pub mod events;

pub use manager::TabManager;
pub use ui::TabUI;
pub use events::TabEvent;

use crate::core::{Tab, BrowserState};
use std::sync::{Arc, Mutex};