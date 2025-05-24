use compositor_utils::Result;

/// Plugin API version
pub const PLUGIN_API_VERSION: u32 = 1;

/// Plugin initialization function signature
pub type PluginInitFn = unsafe extern "C" fn() -> i32;

/// Plugin cleanup function signature  
pub type PluginCleanupFn = unsafe extern "C" fn();

/// Plugin info function signature
pub type PluginInfoFn = unsafe extern "C" fn() -> *const std::os::raw::c_char;

/// Plugin API interface that plugins must implement
pub trait PluginApi {
    /// Initialize the plugin
    fn init(&mut self) -> Result<()>;
    
    /// Cleanup the plugin
    fn cleanup(&mut self);
    
    /// Get plugin information
    fn info(&self) -> &str;
    
    /// Get API version
    fn api_version(&self) -> u32 {
        PLUGIN_API_VERSION
    }
}

/// Plugin capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginCapability {
    /// Can modify window decorations
    WindowDecorations = 1 << 0,
    
    /// Can handle input events
    InputHandling = 1 << 1,
    
    /// Can render to surfaces
    SurfaceRendering = 1 << 2,
    
    /// Can manage workspaces
    WorkspaceManagement = 1 << 3,
    
    /// Can access system resources
    SystemAccess = 1 << 4,
    
    /// Can communicate with external processes
    ExternalCommunication = 1 << 5,
}

/// Plugin context provided to plugins for interacting with the compositor
pub struct PluginContext {
    // TODO: Add compositor interfaces that plugins can use
}

impl PluginContext {
    /// Create a new plugin context
    pub fn new() -> Self {
        Self {}
    }
    
    /// Get the compositor version
    pub fn compositor_version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, _capability: PluginCapability) -> bool {
        // TODO: Implement capability checking
        false
    }
}

impl Default for PluginContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin registration information
#[derive(Debug)]
pub struct PluginRegistration {
    pub name: String,
    pub version: String,
    pub capabilities: Vec<PluginCapability>,
    pub init_fn: PluginInitFn,
    pub cleanup_fn: PluginCleanupFn,
    pub info_fn: PluginInfoFn,
}

impl PluginRegistration {
    /// Create a new plugin registration
    pub fn new(
        name: String,
        version: String,
        capabilities: Vec<PluginCapability>,
        init_fn: PluginInitFn,
        cleanup_fn: PluginCleanupFn,
        info_fn: PluginInfoFn,
    ) -> Self {
        Self {
            name,
            version,
            capabilities,
            init_fn,
            cleanup_fn,
            info_fn,
        }
    }
}
