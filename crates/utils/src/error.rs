use thiserror::Error;

/// Main error type for the compositor
#[derive(Error, Debug)]
pub enum CompositorError {
    #[error("Vulkan error: {0}")]
    Vulkan(#[from] ash::vk::Result),
    
    #[error("Wayland error: {0}")]
    Wayland(String),
    
    #[error("Configuration error: {0}")]
    Config(#[from] ron::error::SpannedError),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("String conversion error: {0}")]
    StringConversion(#[from] std::ffi::NulError),
    
    #[error("Memory allocation error: {0}")]
    Memory(String),
    
    #[error("Plugin error: {0}")]
    Plugin(String),
    
    #[error("Graphics error: {0}")]
    Graphics(String),
    
    #[error("System error: {0}")]
    System(String),
    
    #[error("Initialization error: {0}")]
    Init(String),
    
    #[error("Runtime error: {0}")]
    Runtime(String),
    
    #[error("IPC error: {0}")]
    Ipc(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Backend error: {0}")]
    Backend(String),
}

/// Specialized Result type for compositor operations
pub type Result<T> = std::result::Result<T, CompositorError>;

impl CompositorError {
    /// Create a new Wayland error
    pub fn wayland(msg: impl Into<String>) -> Self {
        Self::Wayland(msg.into())
    }
    
    /// Create a new memory allocation error
    pub fn memory(msg: impl Into<String>) -> Self {
        Self::Memory(msg.into())
    }
    
    /// Create a new plugin error
    pub fn plugin(msg: impl Into<String>) -> Self {
        Self::Plugin(msg.into())
    }
    
    /// Create a new graphics error
    pub fn graphics(msg: impl Into<String>) -> Self {
        Self::Graphics(msg.into())
    }
    
    /// Create a new system error
    pub fn system(msg: impl Into<String>) -> Self {
        Self::System(msg.into())
    }
    
    /// Create a new initialization error
    pub fn init(msg: impl Into<String>) -> Self {
        Self::Init(msg.into())
    }
    
    /// Create a new runtime error
    pub fn runtime(msg: impl Into<String>) -> Self {
        Self::Runtime(msg.into())
    }
    
    /// Create a new IPC error
    pub fn ipc(msg: impl Into<String>) -> Self {
        Self::Ipc(msg.into())
    }
    
    /// Create a new configuration error
    pub fn configuration(msg: impl Into<String>) -> Self {
        Self::Configuration(msg.into())
    }
}
