#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "linux")]
pub use linux::*;

#[cfg(target_os = "windows")]
mod windows {
    // Windows WebView2 implementation will go here
}

#[cfg(target_os = "macos")]
mod macos {
    // macOS WebKit implementation will go here
}

#[cfg(target_os = "linux")]
mod linux {
    // Linux WebKitGTK implementation will go here
}
