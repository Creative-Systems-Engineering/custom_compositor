use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use compositor_utils::{CompositorError, Result};

/// Plugin manifest containing metadata about a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin name
    pub name: String,
    
    /// Plugin version
    pub version: String,
    
    /// Plugin description
    pub description: String,
    
    /// Plugin author
    pub author: String,
    
    /// Minimum compositor version required
    pub min_compositor_version: String,
    
    /// Plugin entry point (shared library file)
    pub entry_point: String,
    
    /// Plugin dependencies
    pub dependencies: Vec<String>,
    
    /// Plugin capabilities/permissions
    pub capabilities: Vec<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl PluginManifest {
    /// Load a plugin manifest from a file
    pub fn load_from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| CompositorError::plugin(format!("Failed to read manifest: {}", e)))?;
        
        let manifest: PluginManifest = ron::from_str(&content)
            .map_err(|e| CompositorError::plugin(format!("Failed to parse manifest: {}", e)))?;
        
        Ok(manifest)
    }
    
    /// Save a plugin manifest to a file
    pub fn save_to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let content = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .map_err(|e| CompositorError::plugin(format!("Failed to serialize manifest: {}", e)))?;
        
        std::fs::write(path, content)
            .map_err(|e| CompositorError::plugin(format!("Failed to write manifest: {}", e)))?;
        
        Ok(())
    }
    
    /// Validate the manifest for correctness
    pub fn validate(&self) -> Result<()> {
        if self.name.is_empty() {
            return Err(CompositorError::plugin("Plugin name cannot be empty".to_string()));
        }
        
        if self.version.is_empty() {
            return Err(CompositorError::plugin("Plugin version cannot be empty".to_string()));
        }
        
        if self.entry_point.is_empty() {
            return Err(CompositorError::plugin("Plugin entry point cannot be empty".to_string()));
        }
        
        // TODO: Add more validation (version format, entry point exists, etc.)
        
        Ok(())
    }
}

impl Default for PluginManifest {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "0.1.0".to_string(),
            description: String::new(),
            author: String::new(),
            min_compositor_version: "0.1.0".to_string(),
            entry_point: "plugin.so".to_string(),
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}
