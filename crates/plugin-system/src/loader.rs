use std::path::PathBuf;
use libloading::{Library, Symbol};
use compositor_utils::{CompositorError, Result};
use crate::api::{PluginInitFn, PluginCleanupFn, PluginInfoFn, PluginRegistration};
use crate::manifest::PluginManifest;

/// Plugin loader for dynamically loading shared libraries
pub struct PluginLoader {
    loaded_libraries: Vec<Library>,
}

impl PluginLoader {
    /// Create a new plugin loader
    pub fn new() -> Self {
        Self {
            loaded_libraries: Vec::new(),
        }
    }
    
    /// Load a plugin from a shared library file
    pub fn load_plugin(&mut self, library_path: &PathBuf, manifest: &PluginManifest) -> Result<PluginRegistration> {
        // Load the shared library
        let library = unsafe {
            Library::new(library_path)
                .map_err(|e| CompositorError::plugin(format!("Failed to load library {}: {}", library_path.display(), e)))?
        };
        
        // Get the required function symbols
        let init_fn: Symbol<PluginInitFn> = unsafe {
            library.get(b"plugin_init\0")
                .map_err(|e| CompositorError::plugin(format!("Failed to find plugin_init symbol: {}", e)))?
        };
        
        let cleanup_fn: Symbol<PluginCleanupFn> = unsafe {
            library.get(b"plugin_cleanup\0")
                .map_err(|e| CompositorError::plugin(format!("Failed to find plugin_cleanup symbol: {}", e)))?
        };
        
        let info_fn: Symbol<PluginInfoFn> = unsafe {
            library.get(b"plugin_info\0")
                .map_err(|e| CompositorError::plugin(format!("Failed to find plugin_info symbol: {}", e)))?
        };
        
        // Create registration
        let registration = PluginRegistration::new(
            manifest.name.clone(),
            manifest.version.clone(),
            Vec::new(), // TODO: Parse capabilities from manifest
            *init_fn,
            *cleanup_fn,
            *info_fn,
        );
        
        // Keep the library loaded
        self.loaded_libraries.push(library);
        
        Ok(registration)
    }
    
    /// Unload all loaded libraries
    pub fn unload_all(&mut self) {
        self.loaded_libraries.clear();
    }
}

impl Default for PluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginLoader {
    fn drop(&mut self) {
        self.unload_all();
    }
}
