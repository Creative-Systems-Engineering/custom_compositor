use std::collections::HashMap;
use std::path::PathBuf;
use crate::manifest::PluginManifest;
use crate::loader::PluginLoader;
use compositor_utils::{CompositorError, Result};

/// Plugin registry for managing loaded plugins
pub struct PluginRegistry {
    plugins: HashMap<String, LoadedPlugin>,
    plugin_paths: Vec<PathBuf>,
    _loader: PluginLoader, // Prefix with _ to indicate intentionally unused for now
}

/// Represents a loaded plugin with its metadata and handle
pub struct LoadedPlugin {
    pub manifest: PluginManifest,
    pub library_path: PathBuf,
    pub is_active: bool,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            plugin_paths: Vec::new(),
            _loader: PluginLoader::new(),
        }
    }
    
    /// Add a plugin search path
    pub fn add_plugin_path(&mut self, path: PathBuf) {
        self.plugin_paths.push(path);
    }
    
    /// Discover plugins in the registered paths
    pub fn discover_plugins(&mut self) -> Result<usize> {
        let discovered = 0; // Remove mut since we're not modifying it yet
        
        for path in &self.plugin_paths {
            if path.is_dir() {
                // TODO: Scan directory for plugin manifests
                // For now, just return success
            }
        }
        
        Ok(discovered)
    }
    
    /// Load a plugin by name
    pub fn load_plugin(&mut self, name: &str) -> Result<()> {
        // TODO: Implement plugin loading logic
        Err(CompositorError::plugin(format!("Plugin loading not yet implemented: {}", name)))
    }
    
    /// Unload a plugin by name
    pub fn unload_plugin(&mut self, name: &str) -> Result<()> {
        if self.plugins.remove(name).is_some() {
            Ok(())
        } else {
            Err(CompositorError::plugin(format!("Plugin not found: {}", name)))
        }
    }
    
    /// Get a list of loaded plugins
    pub fn loaded_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
    
    /// Check if a plugin is loaded
    pub fn is_loaded(&self, name: &str) -> bool {
        self.plugins.contains_key(name)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
