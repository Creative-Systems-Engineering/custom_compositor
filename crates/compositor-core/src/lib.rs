// Compositor Core - Main compositor logic and Wayland protocol handling
//
// This crate implements the core compositor functionality including:
// - Wayland server and protocol handling
// - Window management and layout
// - Input event processing
// - Integration with the Vulkan renderer

use compositor_utils::prelude::*;
use vulkan_renderer::VulkanRenderer;

pub mod wayland;
pub mod window;
pub mod input;
pub mod output;
pub mod surface;
pub mod backend;

use wayland::WaylandServer;
use backend::Backend;

/// Main compositor instance
pub struct Compositor {
    wayland_server: WaylandServer,
    renderer: VulkanRenderer,
    backend: Backend,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Compositor {
    /// Create a new compositor instance
    pub async fn new() -> Result<Self> {
        info!("Initializing custom compositor");
        
        // Initialize renderer first
        let renderer = VulkanRenderer::new()
            .map_err(|e| CompositorError::init(format!("Failed to initialize renderer: {}", e)))?;
        
        info!("Renderer info: {:?}", renderer.get_info());
        
        // Initialize backend (DRM/libinput)
        let backend = Backend::new()
            .await
            .map_err(|e| CompositorError::init(format!("Failed to initialize backend: {}", e)))?;
        
        // Initialize Wayland server
        let mut wayland_server = WaylandServer::new()
            .map_err(|e| CompositorError::init(format!("Failed to initialize Wayland server: {}", e)))?;
        
        // Start listening for client connections
        wayland_server.start_listening()
            .map_err(|e| CompositorError::init(format!("Failed to start Wayland server: {}", e)))?;
        
        let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
        
        info!("Compositor initialized successfully");
        
        Ok(Self {
            wayland_server,
            renderer,
            backend,
            running,
        })
    }
    
    /// Get the Wayland socket name for client connections
    pub fn wayland_socket_name(&self) -> Option<&str> {
        self.wayland_server.socket_name()
    }
    
    /// Start the compositor main loop
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting compositor main loop");
        
        // Setup signal handlers
        self.setup_signal_handlers().await?;
        
        // Main event loop
        while self.running.load(std::sync::atomic::Ordering::Relaxed) {
            // Process backend events (input, output changes, etc.)
            self.backend.process_events().await?;
            
            // Process Wayland events
            self.wayland_server.process_events().await?;
            
            // Render frame
            self.render_frame().await?;
            
            // Small sleep to prevent busy waiting
            tokio::time::sleep(std::time::Duration::from_millis(1)).await;
        }
        
        info!("Compositor main loop ended");
        Ok(())
    }
    
    /// Setup signal handlers for graceful shutdown
    async fn setup_signal_handlers(&self) -> Result<()> {
        let running = self.running.clone();
        
        tokio::spawn(async move {
            let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("Failed to setup SIGTERM handler");
            let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
                .expect("Failed to setup SIGINT handler");
            
            tokio::select! {
                _ = sigterm.recv() => {
                    info!("Received SIGTERM, shutting down");
                }
                _ = sigint.recv() => {
                    info!("Received SIGINT, shutting down");
                }
            }
            
            running.store(false, std::sync::atomic::Ordering::Relaxed);
        });
        
        Ok(())
    }
    
    /// Render a frame
    async fn render_frame(&mut self) -> Result<()> {
        // Begin frame
        self.renderer.begin_frame()?;
        
        // TODO: Render compositor content
        // - Render windows
        // - Render UI elements
        // - Apply effects (glassmorphism, etc.)
        
        // End frame and present
        self.renderer.end_frame()?;
        
        Ok(())
    }
    
    /// Shutdown the compositor
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down compositor");
        
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        
        // Shutdown components in reverse order
        self.wayland_server.shutdown().await?;
        
        info!("Compositor shutdown complete");
        Ok(())
    }
}
