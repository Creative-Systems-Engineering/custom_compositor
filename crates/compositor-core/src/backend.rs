use compositor_utils::prelude::*;
use crate::session::SessionManager;

/// Backend type selection
#[derive(Debug, Clone)]
pub enum BackendType {
    /// Windowed backend (for testing/development)
    Windowed,
    /// DRM backend (for actual compositor)
    Drm,
    /// Auto-detect best backend
    Auto,
}

/// Backend abstraction for different display and input systems
pub struct Backend {
    backend_type: BackendType,
    session_manager: Option<SessionManager>,
}

impl Backend {
    /// Create a new backend with auto-detection
    pub async fn new() -> Result<Self> {
        Self::new_with_type(BackendType::Auto).await
    }
    
    /// Create a new backend with specific type
    pub async fn new_with_type(backend_type: BackendType) -> Result<Self> {
        info!("Initializing backend: {:?}", backend_type);
        
        let actual_type = match backend_type {
            BackendType::Auto => {
                // Try to detect if we can use DRM
                if Self::can_use_drm().await {
                    info!("Auto-detected DRM backend capability");
                    BackendType::Drm
                } else {
                    info!("Falling back to windowed backend");
                    BackendType::Windowed
                }
            }
            other => other,
        };
        
        match actual_type {
            BackendType::Windowed => Self::init_windowed_backend().await,
            BackendType::Drm => Self::init_drm_backend().await,
            BackendType::Auto => unreachable!(),
        }
    }
    
    /// Check if DRM backend is available
    async fn can_use_drm() -> bool {
        // Try to initialize session manager to check seat availability
        match SessionManager::new() {
            Ok(mut session_manager) => {
                match session_manager.initialize() {
                    Ok(()) => {
                        info!("Session manager initialized successfully - DRM backend available");
                        true
                    }
                    Err(e) => {
                        warn!("Session manager initialization failed: {} - falling back to windowed mode", e);
                        false
                    }
                }
            }
            Err(e) => {
                warn!("Could not create session manager: {} - falling back to windowed mode", e);
                false
            }
        }
    }
    
    /// Initialize windowed backend (for development/testing)
    async fn init_windowed_backend() -> Result<Self> {
        info!("Initializing windowed backend");
        
        // TODO: Initialize winit or similar for windowed mode
        // This will be useful for development and testing
        
        Ok(Self {
            backend_type: BackendType::Windowed,
            session_manager: None,
        })
    }
    
    /// Initialize DRM backend (for production compositor)
    async fn init_drm_backend() -> Result<Self> {
        info!("Initializing DRM backend");
        
        // Initialize session manager for secure DRM access
        let mut session_manager = SessionManager::new()?;
        session_manager.initialize()?;
        
        info!("Session manager initialized - waiting for seat activation...");
        
        // Wait a bit for seat activation
        for _ in 0..50 { // Wait up to 500ms
            session_manager.dispatch_events(Some(10))?;
            if session_manager.is_active() {
                info!("Seat activated - DRM device access granted");
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        
        if !session_manager.is_active() {
            return Err(CompositorError::Backend(
                "Failed to activate seat within timeout - compositor cannot access DRM devices".to_string()
            ));
        }
        
        info!("DRM backend initialized successfully with session management");
        
        Ok(Self {
            backend_type: BackendType::Drm,
            session_manager: Some(session_manager),
        })
    }
    
    /// Process backend events (input, output changes, etc.)
    pub async fn process_events(&mut self) -> Result<()> {
        match self.backend_type {
            BackendType::Windowed => self.process_windowed_events().await,
            BackendType::Drm => self.process_drm_events().await,
            BackendType::Auto => unreachable!(),
        }
    }
    
    /// Process events for windowed backend
    async fn process_windowed_events(&mut self) -> Result<()> {
        // TODO: Process winit events
        tokio::task::yield_now().await;
        Ok(())
    }
    
    /// Process events for DRM backend
    async fn process_drm_events(&mut self) -> Result<()> {
        // Process session events to maintain DRM access
        if let Some(ref mut session_manager) = self.session_manager {
            session_manager.dispatch_events(Some(1))?; // Non-blocking check
            
            if !session_manager.is_active() {
                warn!("Session deactivated - compositor paused");
                // In a real implementation, we'd pause rendering and wait for reactivation
            }
        }
        
        // TODO: Process DRM and libinput events
        tokio::task::yield_now().await;
        Ok(())
    }
    
    /// Get backend type
    pub fn backend_type(&self) -> &BackendType {
        &self.backend_type
    }
    
    /// Get DRM file descriptor (if available and active)
    pub fn get_drm_fd(&self) -> Option<std::os::unix::io::RawFd> {
        self.session_manager.as_ref()?.get_drm_fd().ok()
    }
    
    /// Check if session is active
    pub fn is_session_active(&self) -> bool {
        self.session_manager.as_ref()
            .map(|sm| sm.is_active())
            .unwrap_or(false)
    }
}
