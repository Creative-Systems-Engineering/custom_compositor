use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use std::path::Path;

/// Initialize the logging system for the compositor
///
/// This sets up structured logging with:
/// - Console output with colors and formatting
/// - File output for persistent logs
/// - Environment-based log level filtering
/// - JSON formatting for production environments
pub fn setup_logging() -> anyhow::Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,custom_compositor=debug"));

    // Console layer with pretty formatting
    let console_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_level(true)
        .with_ansi(true);

    // File appender for persistent logging
    let log_dir = std::env::var("COMPOSITOR_LOG_DIR")
        .unwrap_or_else(|_| "/tmp/custom_compositor_logs".to_string());
    
    if !Path::new(&log_dir).exists() {
        std::fs::create_dir_all(&log_dir)?;
    }

    let file_appender = tracing_appender::rolling::daily(&log_dir, "compositor.log");
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .json();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .init();

    tracing::info!("Logging system initialized");
    tracing::info!("Log directory: {}", log_dir);
    
    Ok(())
}

/// Setup logging for testing - simplified output
pub fn setup_test_logging() {
    let _ = tracing_subscriber::fmt()
        .with_test_writer()
        .with_env_filter("debug")
        .try_init();
}
