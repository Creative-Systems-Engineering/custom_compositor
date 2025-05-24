// Plugin System - Dynamic plugin loading and management
//
// This crate provides a safe and flexible plugin system for extending
// compositor functionality at runtime.

use compositor_utils::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

pub mod loader;
pub mod registry;
pub mod manifest;
pub mod api;

/// Plugin system manager
pub struct PluginSystem {
    plugins: HashMap<Uuid, LoadedPlugin>,
    _registry: registry::PluginRegistry, // Prefix with _ to indicate intentionally unused for now
}

/// Represents a loaded plugin
pub struct LoadedPlugin {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub enabled: bool,
}

impl PluginSystem {
    /// Create a new plugin system
    pub fn new() -> Result<Self> {
        info!("Initializing Plugin System");
        
        Ok(Self {
            plugins: HashMap::new(),
            _registry: registry::PluginRegistry::new(),
        })
    }
    
    /// Load a plugin from path
    pub async fn load_plugin(&mut self, path: &str) -> Result<Uuid> {
        info!("Loading plugin from: {}", path);
        
        // TODO: Implement actual plugin loading
        let plugin_id = Uuid::new_v4();
        let plugin = LoadedPlugin {
            id: plugin_id,
            name: "Example Plugin".to_string(),
            version: "0.1.0".to_string(),
            enabled: true,
        };
        
        self.plugins.insert(plugin_id, plugin);
        Ok(plugin_id)
    }
    
    /// Unload a plugin
    pub async fn unload_plugin(&mut self, id: Uuid) -> Result<()> {
        if let Some(plugin) = self.plugins.remove(&id) {
            info!("Unloaded plugin: {}", plugin.name);
        }
        Ok(())
    }
    
    /// List all loaded plugins
    pub fn list_plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }
}

impl Default for PluginSystem {
    fn default() -> Self {
        Self::new().expect("Failed to create plugin system")
    }
}
