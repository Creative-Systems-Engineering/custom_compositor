// Custom Wayland Compositor
// High-performance compositor built with Rust and Vulkan for 4K UI/UX development

use compositor_utils::prelude::*;
use compositor_core::Compositor;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging system
    compositor_utils::setup_logging()?;
    
    info!("Starting Custom Wayland Compositor");
    info!("Target: 4K UI/UX development on Debian 12 Linux");
    
    // Print system information
    print_system_info();
    
    // Create and run compositor
    let compositor = Compositor::new().await
        .context("Failed to create compositor")?;
    
    // Display connection information
    if let Some(socket_name) = compositor.wayland_socket_name() {
        info!("Wayland socket available: {}", socket_name);
        info!("Clients can connect with: WAYLAND_DISPLAY={}", socket_name);
    }
    
    info!("Compositor created successfully, starting main loop");
    
    // Run the compositor (this consumes self and handles its own cleanup)
    if let Err(e) = compositor.run().await {
        error!("Compositor error: {}", e);
        return Err(e.into());
    }
    
    info!("Compositor shut down successfully");
    Ok(())
}

fn print_system_info() {
    info!("System Information:");
    
    // Get memory info
    let memory_stats = compositor_utils::memory::get_memory_stats();
    info!("  Memory - Current: {:.2}MB, Peak: {:.2}MB", 
          memory_stats.current_mb(), memory_stats.peak_mb());
    
    // Check for 4K display support
    if let Ok(display_env) = std::env::var("DISPLAY") {
        info!("  Display: {}", display_env);
    }
    
    if let Ok(wayland_display) = std::env::var("WAYLAND_DISPLAY") {
        info!("  Wayland Display: {}", wayland_display);
    }
    
    // Session information
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        info!("  Session Type: {}", session_type);
    }
    
    // Current directory
    if let Ok(current_dir) = std::env::current_dir() {
        info!("  Working Directory: {}", current_dir.display());
    }
    
    // Check for required permissions
    check_permissions();
}

fn check_permissions() {
    // Check if we can access DRM devices
    let drm_paths = ["/dev/dri/card0", "/dev/dri/card1"];
    for path in &drm_paths {
        match std::fs::metadata(path) {
            Ok(_) => info!("  DRM device accessible: {}", path),
            Err(_) => warn!("  DRM device not accessible: {}", path),
        }
    }
    
    // Check for input device access
    match std::fs::read_dir("/dev/input") {
        Ok(entries) => {
            let count = entries.count();
            info!("  Input devices found: {}", count);
        }
        Err(e) => warn!("  Cannot access input devices: {}", e),
    }
}
