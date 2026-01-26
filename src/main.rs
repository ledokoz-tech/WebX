use tracing_subscriber;
use webx::ui::BrowserApp;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    tracing::info!("🌐 WebX Browser - Official system browser for Ledokoz OS");
    tracing::info!("Version 0.1.0");
    tracing::info!("Built with Rust ❤️");
    tracing::info!("Starting browser...");

    // Create and run the browser
    let app = BrowserApp::new()?;
    app.run()?;

    tracing::info!("Browser closed");
    Ok(())
}
