// Tab Events and Communication
use serde::{Deserialize, Serialize};

/// Events that can occur with tabs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabEvent {
    Created { tab_id: usize, url: String },
    Closed { tab_id: usize },
    Activated { tab_id: usize },
    Updated { tab_id: usize, title: Option<String>, url: Option<String> },
    LoadingStarted { tab_id: usize },
    LoadingFinished { tab_id: usize },
    NavigationRequested { tab_id: usize, url: String },
    DuplicateRequested { source_tab_id: usize },
    PinChanged { tab_id: usize, pinned: bool },
    MuteChanged { tab_id: usize, muted: bool },
}

impl TabEvent {
    /// Create a created event
    pub fn created(tab_id: usize, url: String) -> Self {
        Self::Created { tab_id, url }
    }

    /// Create a closed event
    pub fn closed(tab_id: usize) -> Self {
        Self::Closed { tab_id }
    }

    /// Create an activated event
    pub fn activated(tab_id: usize) -> Self {
        Self::Activated { tab_id }
    }

    /// Create an updated event
    pub fn updated(tab_id: usize, title: Option<String>, url: Option<String>) -> Self {
        Self::Updated { tab_id, title, url }
    }

    /// Create a loading started event
    pub fn loading_started(tab_id: usize) -> Self {
        Self::LoadingStarted { tab_id }
    }

    /// Create a loading finished event
    pub fn loading_finished(tab_id: usize) -> Self {
        Self::LoadingFinished { tab_id }
    }
}