// Plugin system modules placeholders
use compositor_utils::prelude::*;

pub mod loader {
    use super::*;
    
    pub struct PluginLoader;
}

pub mod registry {
    use super::*;
    
    pub struct PluginRegistry;
    
    impl PluginRegistry {
        pub fn new() -> Result<Self> {
            Ok(Self)
        }
    }
}

pub mod manifest {
    use super::*;
    
    pub struct PluginManifest {
        pub name: String,
        pub version: String,
    }
}

pub mod api {
    use super::*;
    
    pub trait PluginApi {
        fn name(&self) -> &str;
        fn version(&self) -> &str;
    }
}
